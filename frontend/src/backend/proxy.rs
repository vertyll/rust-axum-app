//! Passes `/api` and `/uploads` on to the back-end, so the browser reaches the
//! back-end through this application's address: the session cookie is set
//! here, every API call is same-origin, and Keycloak returns to
//! `/api/auth/callback` on this address.
//!
//! The request goes on as the browser sent it, its `Sec-Fetch-Site` included,
//! so the back-end's own cross-site check still applies; only the hop-by-hop
//! headers are dropped. Redirects come back to the browser unfollowed.

use reqwest::header::{ACCEPT_LANGUAGE, CONTENT_DISPOSITION, CONTENT_SECURITY_POLICY, HOST, X_CONTENT_TYPE_OPTIONS};
use serde::Serialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::error::not_found;
use topcoat::router::header::{HeaderMap, HeaderName, HeaderValue};
use topcoat::router::request::{headers, method, uri};
use topcoat::router::response::Response;
use topcoat::router::{Body, StatusCode, href, path_param, route};

use super::backend;
use crate::i18n::lang;
use crate::theme::{Theme, take_sign_in_theme, theme};

path_param!(pub *rest);

/// The headers that describe one connection, not the message, and so are
/// not passed on by a proxy (RFC 9110, 7.6.1).
const HOP_BY_HOP: [&str; 8] = [
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

#[route(* "/api/{*rest}")]
pub async fn api(cx: &Cx, body: Body) -> Result<Response> {
    forward(cx, body).await
}

/// The local disk's signed links: the browser sends the bytes of an upload here
/// and downloads files from here. An uploaded file comes from a reader, not from
/// this application, so it is only ever downloaded: no browser renders it on this
/// origin, where a script in it would act with the reader's session.
#[route([GET, HEAD, PUT] "/uploads/{*rest}")]
pub async fn uploads(cx: &Cx, body: Body) -> Result<Response> {
    let mut response = forward(cx, body).await?;
    let response_headers = response.headers_mut();
    response_headers
        .entry(CONTENT_DISPOSITION)
        .or_insert(HeaderValue::from_static("attachment"));
    response_headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    response_headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    Ok(response)
}

pub fn sign_in(cx: &Cx) -> String {
    href!(api, Rest(["auth", "authorize"])).resolve(cx)
}

pub fn register(cx: &Cx) -> String {
    #[derive(Serialize)]
    struct Register {
        register: bool,
    }
    href!(api, Rest(["auth", "authorize"]))
        .query(Register { register: true })
        .resolve(cx)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeycloakAction {
    UpdatePassword,
    ConfigureTotp,
    /// Removes the credential with this id, after Keycloak asks the reader to confirm.
    DeleteCredential(String),
}

pub fn keycloak_action(cx: &Cx, action: &KeycloakAction) -> String {
    #[derive(Serialize)]
    struct Action {
        kc_action: String,
    }
    let kc_action = match action {
        KeycloakAction::UpdatePassword => "UPDATE_PASSWORD".to_owned(),
        KeycloakAction::ConfigureTotp => "CONFIGURE_TOTP".to_owned(),
        KeycloakAction::DeleteCredential(id) => format!("delete_credential:{id}"),
    };
    href!(api, Rest(["auth", "authorize"]))
        .query(Action { kc_action })
        .resolve(cx)
}

async fn forward(cx: &Cx, body: Body) -> Result<Response> {
    // The URL parser would resolve `..`, reaching paths outside the prefix.
    if has_dot_segment(uri(cx).path()) {
        return Err(not_found().into());
    }
    let mut path_and_query = uri(cx).path_and_query().map_or("/", |path| path.as_str()).to_owned();
    let mut request_headers = passed_on(headers(cx));
    // Keycloak shows its pages in the language and the theme the reader chose here.
    if uri(cx).path() == "/api/auth/authorize" {
        request_headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static(lang(cx).code()));
        path_and_query = with_theme(&path_and_query, theme(cx));
    }
    if uri(cx).path() == "/api/auth/callback" {
        take_sign_in_theme(cx, headers(cx).get(HOST).and_then(|host| host.to_str().ok()));
    }

    let backend = backend(cx);
    let sent = backend
        .proxy
        .request(method(cx).clone(), backend.url(&path_and_query))
        .headers(request_headers)
        .body(reqwest::Body::wrap_stream(body.into_data_stream()))
        .send()
        .await;
    let response = match sent {
        Ok(response) => response,
        Err(err) => {
            tracing::error!("the back-end did not answer the proxy: {err}");
            let status = if err.is_timeout() {
                StatusCode::GATEWAY_TIMEOUT
            } else {
                StatusCode::BAD_GATEWAY
            };
            return Ok(Response::builder()
                .status(status)
                .body(Body::empty())
                .expect("a status and an empty body make a response"));
        }
    };

    let mut response: Response<reqwest::Body> = response.into();
    let response_headers = passed_on(response.headers());
    *response.headers_mut() = response_headers;
    Ok(response.map(Body::new))
}

/// The path and query with `theme` set to the reader's, in place of any the request carried.
fn with_theme(path_and_query: &str, theme: Theme) -> String {
    let (path, query) = path_and_query.split_once('?').unwrap_or((path_and_query, ""));
    let mut pairs = form_urlencoded::Serializer::new(String::new());
    for (name, value) in form_urlencoded::parse(query.as_bytes()).filter(|(name, _)| name != "theme") {
        pairs.append_pair(&name, &value);
    }
    pairs.append_pair("theme", theme.code());
    format!("{path}?{}", pairs.finish())
}

fn has_dot_segment(path: &str) -> bool {
    path.split('/').any(|segment| {
        let segment = segment.to_ascii_lowercase().replace("%2e", ".");
        segment == "." || segment == ".."
    })
}

/// The headers a proxy passes on: all but the hop-by-hop ones, those the
/// `Connection` header names, and `Host`, which belongs to the next hop.
fn passed_on(headers: &HeaderMap) -> HeaderMap {
    let named_by_connection: Vec<String> = headers
        .get_all("connection")
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .map(|name| name.trim().to_ascii_lowercase())
        .collect();
    headers
        .iter()
        .filter(|(name, _)| {
            let name = name.as_str();
            name != HOST.as_str()
                && !HOP_BY_HOP.contains(&name)
                && !named_by_connection.iter().any(|named| named == name)
        })
        .map(|(name, value)| (HeaderName::clone(name), HeaderValue::clone(value)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::with_theme;
    use crate::theme::Theme;

    #[test]
    fn the_theme_of_the_reader_replaces_any_the_request_carried() {
        assert_eq!(
            with_theme("/api/auth/authorize", Theme::Dark),
            "/api/auth/authorize?theme=dark"
        );
        assert_eq!(
            with_theme("/api/auth/authorize?register=true&theme=light", Theme::System),
            "/api/auth/authorize?register=true&theme=system"
        );
    }

    use topcoat::router::header::{HeaderMap, HeaderValue};

    use super::{has_dot_segment, passed_on};

    #[test]
    fn drops_hop_by_hop_headers_and_those_connection_names() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("localhost:3000"));
        headers.insert("connection", HeaderValue::from_static("keep-alive, x-trace"));
        headers.insert("keep-alive", HeaderValue::from_static("timeout=5"));
        headers.insert("x-trace", HeaderValue::from_static("1"));
        headers.insert("cookie", HeaderValue::from_static("RUST_AXUM_APP_SESSION=abc"));
        headers.insert("sec-fetch-site", HeaderValue::from_static("same-origin"));

        let passed = passed_on(&headers);

        assert_eq!(passed.len(), 2);
        assert_eq!(passed["cookie"], "RUST_AXUM_APP_SESSION=abc");
        assert_eq!(passed["sec-fetch-site"], "same-origin");
    }

    #[test]
    fn a_dot_segment_cannot_leave_the_prefix() {
        assert!(has_dot_segment("/api/../health"));
        assert!(has_dot_segment("/api/%2E%2e/health"));
        assert!(has_dot_segment("/api/./files"));
        assert!(!has_dot_segment("/api/admin/translations/home.welcome"));
    }
}
