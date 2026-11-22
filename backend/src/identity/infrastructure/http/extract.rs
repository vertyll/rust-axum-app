//! HTTP authentication. [`authenticate`] is a generic middleware layered by
//! the composition root onto protected routers (also of other modules); it
//! takes the access token from `Authorization: Bearer` or, for a browser,
//! from its session, refreshing it when it is about to expire, after the
//! session's CSRF token was checked. [`Auth`] and [`Permitted`] read the
//! [`Caller`] it stores in the request extensions.

use std::marker::PhantomData;
use std::sync::Arc;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use jiff::{SignedDuration, Timestamp};
use tower_cookies::Cookies;
use tower_sessions::Session;

use super::csrf::{self, CsrfCookie};
use super::error::ApiError;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::application::{AuthSession, Caller};
use crate::identity::domain::{IdentityError, Permission};

pub(crate) const SESSION_KEY: &str = "identity.session";
const REFRESH_SKEW: SignedDuration = SignedDuration::from_secs(30);
const FETCH_SITE: &str = "sec-fetch-site";
const BEARER: &str = "Bearer";

/// What the [`authenticate`] layer needs: the service that verifies tokens,
/// and how the session's CSRF token reaches the browser.
pub struct Authentication<P: IdentityPorts> {
    identity: Arc<IdentityService<P>>,
    csrf_cookie: CsrfCookie,
}

impl<P: IdentityPorts> Authentication<P> {
    #[must_use]
    pub fn new(identity: Arc<IdentityService<P>>, csrf_cookie: CsrfCookie) -> Self {
        Self { identity, csrf_cookie }
    }
}

impl<P: IdentityPorts> Clone for Authentication<P> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            csrf_cookie: self.csrf_cookie,
        }
    }
}

/// A request with `Authorization: Bearer` is authenticated by its token alone,
/// which no other site can make a browser send. A request authenticated by the
/// session must also come from this origin and, when unsafe, carry the
/// session's CSRF token.
pub async fn authenticate<P: IdentityPorts>(
    State(authentication): State<Authentication<P>>,
    session: Session,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let token = match bearer_token(request.headers()) {
        Some(token) => token.to_owned(),
        None if sent_from_this_origin(request.method(), request.headers()) => {
            let current = signed_in(&session).await?.ok_or(IdentityError::Unauthenticated)?;
            csrf::verify(request.method(), request.headers(), &session).await?;
            let token = session_token(&authentication.identity, &session, current)
                .await?
                .ok_or(IdentityError::Unauthenticated)?;
            authentication.csrf_cookie.issue(&session, &cookies).await?;
            token
        }
        None => return Err(IdentityError::Unauthenticated.into()),
    };
    let caller = authentication.identity.authenticate(&token).await?;
    request.extensions_mut().insert(caller);
    request.extensions_mut().insert(CallerToken(token));
    Ok(next.run(request).await)
}

/// The access token the [`authenticate`] layer verified, for a call Keycloak answers
/// about the caller themselves. Requires the layer.
#[derive(Clone)]
pub struct CallerToken(pub String);

impl<S: Send + Sync> FromRequestParts<S> for CallerToken {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<CallerToken>()
            .cloned()
            .ok_or_else(|| IdentityError::Unauthenticated.into())
    }
}

pub(crate) async fn signed_in(session: &Session) -> Result<Option<AuthSession>, IdentityError> {
    session.get::<AuthSession>(SESSION_KEY).await.map_err(store)
}

async fn session_token<P: IdentityPorts>(
    identity: &IdentityService<P>,
    session: &Session,
    current: AuthSession,
) -> Result<Option<String>, IdentityError> {
    if !current.needs_refresh_at(Timestamp::now(), REFRESH_SKEW) {
        return Ok(Some(current.access_token));
    }
    match identity.refresh(&current).await {
        Ok(refreshed) => {
            session.insert(SESSION_KEY, &refreshed).await.map_err(store)?;
            Ok(Some(refreshed.access_token))
        }
        Err(IdentityError::SessionExpired) => {
            session.flush().await.map_err(store)?;
            Ok(None)
        }
        Err(err) => {
            tracing::warn!(keycloak_id = %current.identity.keycloak_id, "could not refresh the session: {err}");
            Ok(None)
        }
    }
}

/// A cross-site write is not given the session's token and cannot sign out:
/// `SameSite=Lax` keeps the cookie off cross-site requests, and this also covers
/// sibling subdomains.
pub(crate) fn sent_from_this_origin(method: &Method, headers: &HeaderMap) -> bool {
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return true;
    }
    headers
        .get(FETCH_SITE)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|site| site == "same-origin" || site == "none")
}

/// The access token of `Authorization: Bearer <token>`, the one test of
/// whether a request is authenticated by a token rather than the session. The
/// scheme is matched without regard to case (RFC 9110, 11.1); a header of
/// another scheme, or whose token is not a `b64token` (RFC 6750, 2.1), is none.
pub(crate) fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let (scheme, token) = headers.get(header::AUTHORIZATION)?.to_str().ok()?.split_once(' ')?;
    let token = token.trim_start_matches(' ');
    (scheme.eq_ignore_ascii_case(BEARER) && is_b64token(token)).then_some(token)
}

/// `1*( ALPHA / DIGIT / "-" / "." / "_" / "~" / "+" / "/" ) *"="`
fn is_b64token(token: &str) -> bool {
    let value = token.trim_end_matches('=');
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._~+/".contains(&byte))
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "passed to `map_err`, which hands over the error it maps"
)]
pub(crate) fn store(err: tower_sessions::session::Error) -> IdentityError {
    IdentityError::SessionStoreFailure(err.to_string())
}

/// The authenticated caller. Requires the [`authenticate`] layer.
pub struct Auth(pub Caller);

impl<S: Send + Sync> FromRequestParts<S> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Caller>()
            .cloned()
            .map(Auth)
            .ok_or_else(|| IdentityError::Unauthenticated.into())
    }
}

/// Like [`Auth`], but the caller's roles must grant the permission `P`:
/// `Permitted(caller, _): Permitted<FilesManage>`.
pub struct Permitted<P: Permission>(pub Caller, pub PhantomData<fn() -> P>);

impl<S: Send + Sync, P: Permission> FromRequestParts<S> for Permitted<P> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Auth(caller) = Auth::from_request_parts(parts, state).await?;
        if !caller.may::<P>() {
            return Err(IdentityError::PermissionDenied(P::NAME).into());
        }
        Ok(Permitted(caller, PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, Method, header};

    use super::{bearer_token, sent_from_this_origin};

    fn authorization(value: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static(value));
        headers
    }

    #[test]
    fn extracts_the_token_whatever_the_case_of_the_scheme() {
        for value in ["Bearer abc.def", "BEARER abc.def", "bEaReR abc.def", "bearer  abc.def"] {
            assert_eq!(bearer_token(&authorization(value)), Some("abc.def"), "{value:?}");
        }
        assert_eq!(
            bearer_token(&authorization("Bearer a-b_c~d+e/f==")),
            Some("a-b_c~d+e/f==")
        );
    }

    #[test]
    fn ignores_absence_and_other_schemes() {
        assert_eq!(bearer_token(&HeaderMap::new()), None);
        assert_eq!(bearer_token(&authorization("Basic YWRhOnNlY3JldA==")), None);
        assert_eq!(bearer_token(&authorization("BearerX abc")), None);
    }

    #[test]
    fn ignores_a_malformed_bearer_header() {
        for value in [
            "Bearer",
            "Bearer ",
            "Bearerabc",
            "Bearer abc def",
            "Bearer ==",
            "Bearer a=b",
            " Bearer abc",
        ] {
            assert_eq!(bearer_token(&authorization(value)), None, "{value:?}");
        }
    }

    #[test]
    fn only_cross_site_writes_lose_the_session_token() {
        let mut cross_site = HeaderMap::new();
        cross_site.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
        assert!(sent_from_this_origin(&Method::GET, &cross_site));
        assert!(!sent_from_this_origin(&Method::POST, &cross_site));
        assert!(sent_from_this_origin(&Method::POST, &HeaderMap::new()));

        let mut same_site = HeaderMap::new();
        same_site.insert("sec-fetch-site", HeaderValue::from_static("same-site"));
        assert!(!sent_from_this_origin(&Method::POST, &same_site));
    }
}
