//! The back-end's REST API, as this server calls it on the reader's behalf.
//! Each feature keeps its own calls in its `api` module; this one holds what
//! they share.
//!
//! Every call carries the browser's session cookie, so the back-end authorizes
//! it exactly as it would a call from the browser, and every `Set-Cookie` the
//! back-end answers with is passed on to the browser. The back-end never sees
//! a `Sec-Fetch-Site` header from here; the origin of the browser's own
//! request was already checked by the router's origin policy.

mod problem;
pub mod proxy;

use std::time::Duration;

use reqwest::header::{COOKIE, SET_COOKIE};
use reqwest::{Method, RequestBuilder, Response, StatusCode, redirect};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, cookies};
use topcoat::router::error::{forbidden, internal_server_error, not_found, unauthorized};

pub use problem::Problem;

const SESSION_COOKIE: &str = "RUST_AXUM_APP_SESSION";

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("the back-end could not be reached: {0}")]
    Unreachable(#[source] reqwest::Error),
    #[error("the back-end's answer could not be read: {0}")]
    Malformed(#[source] reqwest::Error),
    #[error("the back-end refused the request with {}: {}", .0.status, .0.code)]
    Refused(Problem),
    #[error("the back-end answered {0} without a problem document")]
    Unexpected(StatusCode),
}

impl BackendError {
    /// The response a page gives when the data it shows cannot be read: the
    /// router's own error for a status it knows, and `500` for the rest.
    pub fn into_page_error(self) -> topcoat::Error {
        match self.status() {
            Some(StatusCode::UNAUTHORIZED) => unauthorized().into(),
            Some(StatusCode::FORBIDDEN) => forbidden().into(),
            Some(StatusCode::NOT_FOUND) => not_found().into(),
            _ => internal_server_error(self).into(),
        }
    }

    #[must_use]
    pub fn status(&self) -> Option<StatusCode> {
        match self {
            BackendError::Unreachable(_) | BackendError::Malformed(_) => None,
            BackendError::Refused(problem) => Some(problem.status),
            BackendError::Unexpected(status) => Some(*status),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct PageRequest {
    pub page: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    #[serde(rename = "page")]
    pub number: u32,
    pub size: u32,
    pub total_elements: u64,
    pub total_pages: u64,
}

#[derive(Debug)]
pub struct Backend {
    /// For API calls: each must finish within the configured time.
    api: reqwest::Client,
    /// For the proxy: a transfer may take long, but must not stall.
    proxy: reqwest::Client,
    base_url: String,
}

impl Backend {
    pub fn new(base_url: &str, timeout: Duration) -> Result<Self, reqwest::Error> {
        let api = reqwest::Client::builder()
            .timeout(timeout)
            .redirect(redirect::Policy::none())
            .build()?;
        let proxy = reqwest::Client::builder()
            .connect_timeout(timeout)
            .read_timeout(timeout)
            .redirect(redirect::Policy::none())
            .build()?;
        Ok(Self {
            api,
            proxy,
            base_url: base_url.trim_end_matches('/').to_owned(),
        })
    }

    fn url(&self, path_and_query: &str) -> String {
        format!("{}{path_and_query}", self.base_url)
    }
}

fn backend(cx: &Cx) -> &Backend {
    app_context::<Backend>(cx)
}

pub fn request(cx: &Cx, method: Method, path: &str) -> RequestBuilder {
    let builder = public_request(cx, method, path);
    match cookies(cx).get(SESSION_COOKIE) {
        Some(session) => builder.header(COOKIE, format!("{SESSION_COOKIE}={}", session.value())),
        None => builder,
    }
}

pub fn public_request(cx: &Cx, method: Method, path: &str) -> RequestBuilder {
    let backend = backend(cx);
    backend.api.request(method, backend.url(path))
}

pub async fn send(cx: &Cx, request: RequestBuilder) -> Result<Response, BackendError> {
    let response = request.send().await.map_err(BackendError::Unreachable)?;
    relay_cookies(cx, &response);
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    match response.json::<Problem>().await {
        Ok(problem) => Err(BackendError::Refused(problem)),
        Err(_) => Err(BackendError::Unexpected(status)),
    }
}

pub async fn json<T: DeserializeOwned>(cx: &Cx, request: RequestBuilder) -> Result<T, BackendError> {
    send(cx, request).await?.json().await.map_err(BackendError::Malformed)
}

pub fn segment(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => char::from(byte).to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

/// Passes each `Set-Cookie` of the back-end on to the browser, such as the
/// session cookie's renewed expiry or its removal at sign-out.
fn relay_cookies(cx: &Cx, response: &Response) {
    let jar = cookies(cx);
    response
        .headers()
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .filter_map(|value| Cookie::parse(value.to_owned()).ok())
        .for_each(|cookie| jar.add(cookie));
}

#[cfg(test)]
mod tests {
    use super::segment;

    #[test]
    fn a_segment_cannot_reach_another_path() {
        assert_eq!(segment("editor"), "editor");
        assert_eq!(segment("../users"), "..%2Fusers");
        assert_eq!(segment("a b?c"), "a%20b%3Fc");
    }
}
