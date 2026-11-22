//! Tokens against cross-site request forgery, in the contract a single-page
//! client of a Spring Security API expects: the session holds a random token,
//! the browser gets it in the cookie `XSRF-TOKEN`, which scripts may read, and
//! every unsafe request the session authenticates sends it back in the header
//! `X-XSRF-TOKEN`. The token is compared with the session's, not with the
//! cookie, so a cookie planted by a sibling subdomain is refused.

use axum::http::{HeaderMap, HeaderValue, Method};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngExt;
use subtle::ConstantTimeEq;
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use tower_sessions::Session;

use super::error::ApiError;
use super::extract::store;
use crate::identity::domain::IdentityError;

pub const COOKIE_NAME: &str = "XSRF-TOKEN";
pub const HEADER_NAME: &str = "x-xsrf-token";
const SESSION_KEY: &str = "identity.csrf_token";
const TOKEN_BYTES: usize = 32;

/// How the token's cookie is written. Scripts of the client read it, so it is
/// not `HttpOnly`; it is `Secure` exactly where the session cookie is, and
/// ends with the browser, as the next request the session authenticates
/// writes it again.
#[derive(Debug, Clone, Copy)]
pub struct CsrfCookie {
    secure: bool,
}

impl CsrfCookie {
    #[must_use]
    pub fn new(secure: bool) -> Self {
        Self { secure }
    }

    /// Gives the session a new token and the browser its cookie, as a sign-in
    /// does, so a token known before it is worthless.
    pub(crate) async fn rotate(self, session: &Session, cookies: &Cookies) -> Result<(), IdentityError> {
        let token = new_token();
        session.insert(SESSION_KEY, &token).await.map_err(store)?;
        cookies.add(self.cookie(token));
        Ok(())
    }

    /// Writes the session's token to the browser when its cookie holds another
    /// or none, and gives a session without one its first.
    pub(crate) async fn issue(self, session: &Session, cookies: &Cookies) -> Result<(), IdentityError> {
        let Some(token) = session.get::<String>(SESSION_KEY).await.map_err(store)? else {
            return self.rotate(session, cookies).await;
        };
        if cookies.get(COOKIE_NAME).is_none_or(|sent| sent.value() != token) {
            cookies.add(self.cookie(token));
        }
        Ok(())
    }

    /// Removes the cookie from the browser, as a sign-out does.
    pub(crate) fn clear(self, cookies: &Cookies) {
        cookies.remove(self.cookie(String::new()));
    }

    fn cookie(self, token: String) -> Cookie<'static> {
        Cookie::build((COOKIE_NAME, token))
            .path("/")
            .same_site(SameSite::Lax)
            .secure(self.secure)
            .http_only(false)
            .build()
    }
}

/// Lets a safe request through, and an unsafe one only when its
/// `X-XSRF-TOKEN` is the session's token.
pub(crate) async fn verify(method: &Method, headers: &HeaderMap, session: &Session) -> Result<(), ApiError> {
    if method.is_safe() {
        return Ok(());
    }
    let expected = session.get::<String>(SESSION_KEY).await.map_err(store)?;
    if matches(expected.as_deref(), headers.get(HEADER_NAME)) {
        Ok(())
    } else {
        Err(ApiError::CsrfTokenRejected)
    }
}

/// Compares in time independent of where the values first differ.
fn matches(expected: Option<&str>, sent: Option<&HeaderValue>) -> bool {
    match (expected, sent) {
        (Some(expected), Some(sent)) => expected.as_bytes().ct_eq(sent.as_bytes()).into(),
        _ => false,
    }
}

fn new_token() -> String {
    let mut buffer = [0u8; TOKEN_BYTES];
    rand::rng().fill(&mut buffer[..]);
    URL_SAFE_NO_PAD.encode(buffer)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;
    use tower_cookies::cookie::SameSite;

    use super::{COOKIE_NAME, CsrfCookie, matches, new_token};

    #[test]
    fn only_the_sessions_own_token_matches() {
        let token = new_token();
        assert!(matches(Some(&token), Some(&HeaderValue::from_str(&token).unwrap())));
        assert!(!matches(Some(&token), Some(&HeaderValue::from_static("forged"))));
        assert!(!matches(Some(&token), Some(&HeaderValue::from_static(""))));
        assert!(!matches(Some(&token), None));
        assert!(!matches(None, Some(&HeaderValue::from_str(&token).unwrap())));
    }

    #[test]
    fn every_token_is_new_and_carries_256_bits() {
        let token = new_token();
        assert_ne!(token, new_token());
        assert_eq!(token.len(), 43);
    }

    #[test]
    fn the_cookie_is_readable_by_scripts_and_secure_where_the_session_is() {
        let cookie = CsrfCookie::new(true).cookie("token".into());
        assert_eq!(cookie.name(), COOKIE_NAME);
        assert_eq!(cookie.path(), Some("/"));
        assert_eq!(cookie.same_site(), Some(SameSite::Lax));
        assert_eq!(cookie.secure(), Some(true));
        assert_eq!(cookie.http_only(), Some(false));
        assert_eq!(cookie.domain(), None);
        assert_eq!(cookie.max_age(), None);

        assert_eq!(CsrfCookie::new(false).cookie("token".into()).secure(), Some(false));
    }

    mod after_sign_in {
        use std::sync::Arc;

        use axum::Router;
        use axum::body::Body;
        use axum::http::{Request, StatusCode, header};
        use axum::middleware::from_fn_with_state;
        use axum::response::Response;
        use axum::routing::get;
        use secrecy::SecretString;
        use tower::ServiceExt;
        use tower_cookies::Cookie;
        use tower_cookies::cookie::SameSite;
        use tower_sessions::{MemoryStore, SessionManagerLayer};

        use super::super::{COOKIE_NAME, CsrfCookie, HEADER_NAME};
        use crate::identity::application::testing::{Harness, TestPorts};
        use crate::identity::infrastructure::http::{Authentication, SignInSettings, auth_router, authenticate};
        use crate::identity::infrastructure::keycloak::KeycloakSettings;

        const SESSION: &str = "SESSION";

        /// The sign-in routes and a protected route, behind sessions kept in memory.
        async fn app() -> Router {
            let identity = Arc::new(Harness::new().await.service);
            let csrf_cookie = CsrfCookie::new(true);
            let settings = SignInSettings {
                keycloak: KeycloakSettings {
                    realm_url: "http://keycloak.test/realms/app".into(),
                    client_id: "app".into(),
                    client_secret: SecretString::from("secret"),
                    audience: "app".into(),
                    callback_url: "http://localhost:3000/api/auth/callback".into(),
                },
                post_login_url: "http://localhost:3000/".into(),
                csrf_cookie,
            };
            let protected = Router::new()
                .route(
                    "/probe",
                    get(|| async { StatusCode::NO_CONTENT }).post(|| async { StatusCode::NO_CONTENT }),
                )
                .layer(from_fn_with_state(
                    Authentication::new(identity.clone(), csrf_cookie),
                    authenticate::<TestPorts>,
                ));
            Router::new()
                .nest("/api/auth", auth_router(identity, settings))
                .merge(protected)
                .layer(SessionManagerLayer::new(MemoryStore::default()).with_name(SESSION))
        }

        fn set_cookie(response: &Response, name: &str) -> Option<Cookie<'static>> {
            response
                .headers()
                .get_all(header::SET_COOKIE)
                .iter()
                .filter_map(|value| Cookie::parse(value.to_str().ok()?.to_owned()).ok())
                .find(|cookie| cookie.name() == name)
        }

        /// A browser's cookies after a sign-in.
        struct Browser {
            session: String,
            token: String,
            token_cookie: Cookie<'static>,
        }

        impl Browser {
            fn cookies(&self) -> String {
                format!("{SESSION}={}; {COOKIE_NAME}={}", self.session, self.token)
            }
        }

        async fn sign_in(app: &Router) -> Browser {
            let authorize = app
                .clone()
                .oneshot(Request::get("/api/auth/authorize").body(Body::empty()).unwrap())
                .await
                .unwrap();
            let before = set_cookie(&authorize, SESSION).unwrap();
            let location = authorize.headers()[header::LOCATION].to_str().unwrap();
            let state = url::Url::parse(location)
                .unwrap()
                .query_pairs()
                .find(|(name, _)| name == "state")
                .unwrap()
                .1
                .into_owned();

            let callback = app
                .clone()
                .oneshot(
                    Request::get(format!("/api/auth/callback?code=code&state={state}"))
                        .header(header::COOKIE, format!("{SESSION}={}", before.value()))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(callback.status(), StatusCode::SEE_OTHER);
            let session = set_cookie(&callback, SESSION).unwrap();
            assert_ne!(session.value(), before.value());
            let token_cookie = set_cookie(&callback, COOKIE_NAME).unwrap();
            Browser {
                session: session.value().to_owned(),
                token: token_cookie.value().to_owned(),
                token_cookie,
            }
        }

        async fn send(app: &Router, request: axum::http::request::Builder) -> Response {
            app.clone().oneshot(request.body(Body::empty()).unwrap()).await.unwrap()
        }

        async fn problem_code(response: Response) -> String {
            assert_eq!(response.headers()[header::CONTENT_TYPE], "application/problem+json");
            let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
            let problem: serde_json::Value = serde_json::from_slice(&body).unwrap();
            problem["code"].as_str().unwrap().to_owned()
        }

        #[tokio::test]
        async fn the_sign_in_writes_a_cookie_scripts_read_and_other_sites_do_not_get() {
            let browser = sign_in(&app().await).await;
            let cookie = &browser.token_cookie;
            assert_eq!(cookie.value().len(), 43);
            assert_eq!(cookie.path(), Some("/"));
            assert_eq!(cookie.same_site(), Some(SameSite::Lax));
            assert_eq!(cookie.secure(), Some(true));
            assert_ne!(cookie.http_only(), Some(true));
            assert_eq!(cookie.domain(), None);
        }

        #[tokio::test]
        async fn a_write_with_the_sessions_token_passes() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/probe")
                    .header(header::COOKIE, browser.cookies())
                    .header(HEADER_NAME, &browser.token),
            )
            .await;
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
        }

        #[tokio::test]
        async fn a_write_without_the_token_is_forbidden_with_a_problem_document() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(&app, Request::post("/probe").header(header::COOKIE, browser.cookies())).await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(problem_code(response).await, "auth.errors.csrf_token_invalid");
        }

        #[tokio::test]
        async fn a_write_with_another_token_is_forbidden() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/probe")
                    .header(header::COOKIE, browser.cookies())
                    .header(HEADER_NAME, "forged"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn a_cookie_planted_by_a_sibling_subdomain_does_not_pass() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/probe")
                    .header(
                        header::COOKIE,
                        format!("{SESSION}={}; {COOKIE_NAME}=planted", browser.session),
                    )
                    .header(HEADER_NAME, "planted"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn a_token_known_before_a_sign_in_is_worthless_after_it() {
            let app = app().await;
            let first = sign_in(&app).await;
            let second = sign_in(&app).await;
            assert_ne!(first.token, second.token);
            let response = send(
                &app,
                Request::post("/probe")
                    .header(header::COOKIE, second.cookies())
                    .header(HEADER_NAME, &first.token),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn a_safe_request_needs_no_token_and_gets_the_cookie_back_when_it_lacks_it() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::get("/probe").header(header::COOKIE, format!("{SESSION}={}", browser.session)),
            )
            .await;
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
            assert_eq!(set_cookie(&response, COOKIE_NAME).unwrap().value(), browser.token);

            let response = send(&app, Request::get("/probe").header(header::COOKIE, browser.cookies())).await;
            assert_eq!(response.status(), StatusCode::NO_CONTENT);
            assert!(set_cookie(&response, COOKIE_NAME).is_none());
        }

        #[tokio::test]
        async fn a_bearer_token_needs_no_csrf_token_whatever_the_case_of_the_scheme() {
            let app = app().await;
            for authorization in ["Bearer access", "bearer access", "BEARER access", "bEaReR access"] {
                let response = send(
                    &app,
                    Request::post("/probe").header(header::AUTHORIZATION, authorization),
                )
                .await;
                assert_eq!(response.status(), StatusCode::NO_CONTENT, "{authorization:?}");
            }
        }

        #[tokio::test]
        async fn a_malformed_bearer_header_with_the_session_still_needs_the_token() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/probe")
                    .header(header::AUTHORIZATION, "Bearer ")
                    .header(header::COOKIE, browser.cookies()),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
        }

        #[tokio::test]
        async fn another_scheme_with_the_session_still_needs_the_token() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let basic = || {
                Request::post("/probe")
                    .header(header::AUTHORIZATION, "Basic YWRhOnNlY3JldA==")
                    .header(header::COOKIE, browser.cookies())
            };
            assert_eq!(send(&app, basic()).await.status(), StatusCode::FORBIDDEN);
            assert_eq!(
                send(&app, basic().header(HEADER_NAME, &browser.token)).await.status(),
                StatusCode::NO_CONTENT
            );
        }

        #[tokio::test]
        async fn a_write_without_a_session_is_unauthenticated_rather_than_forbidden() {
            let response = send(&app().await, Request::post("/probe")).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn a_cross_site_write_is_refused_even_with_the_token() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/probe")
                    .header(header::COOKIE, browser.cookies())
                    .header(HEADER_NAME, &browser.token)
                    .header("sec-fetch-site", "same-site"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }

        #[tokio::test]
        async fn a_cross_site_sign_out_is_forbidden_with_a_problem_document() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let response = send(
                &app,
                Request::post("/api/auth/logout")
                    .header(header::COOKIE, browser.cookies())
                    .header(HEADER_NAME, &browser.token)
                    .header("sec-fetch-site", "cross-site"),
            )
            .await;
            assert_eq!(response.status(), StatusCode::FORBIDDEN);
            assert_eq!(problem_code(response).await, "auth.errors.cross_site_request");

            let still_signed_in = send(&app, Request::get("/probe").header(header::COOKIE, browser.cookies())).await;
            assert_eq!(still_signed_in.status(), StatusCode::NO_CONTENT);
        }

        #[tokio::test]
        async fn a_sign_out_needs_the_token_and_removes_its_cookie() {
            let app = app().await;
            let browser = sign_in(&app).await;
            let logout = || Request::post("/api/auth/logout").header(header::COOKIE, browser.cookies());

            let refused = send(&app, logout()).await;
            assert_eq!(refused.status(), StatusCode::FORBIDDEN);
            assert_eq!(problem_code(refused).await, "auth.errors.csrf_token_invalid");

            let signed_out = send(&app, logout().header(HEADER_NAME, &browser.token)).await;
            assert_eq!(signed_out.status(), StatusCode::NO_CONTENT);
            let removed = set_cookie(&signed_out, COOKIE_NAME).unwrap();
            assert_eq!(removed.value(), "");
            assert_eq!(removed.max_age(), Some(tower_cookies::cookie::time::Duration::ZERO));
        }
    }
}
