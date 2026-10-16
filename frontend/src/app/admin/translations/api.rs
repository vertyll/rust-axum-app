use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

use crate::backend::{self, BackendError, Page, PageRequest, segment};

#[derive(Debug, Clone, Deserialize)]
pub struct Translation {
    pub key: String,
    pub message: LocalizedText,
    pub default: LocalizedText,
    /// Whether an administrator's text replaces the default.
    pub customized: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalizedText {
    pub pl: String,
    pub en: String,
}

#[derive(Debug, Serialize)]
pub struct TextChanges<'a> {
    pub pl: &'a str,
    pub en: &'a str,
}

pub async fn list(cx: &Cx, filter: &str, request: PageRequest) -> Result<Page<Translation>, BackendError> {
    let query = backend::request(cx, Method::GET, "/api/admin/translations")
        .query(&request)
        .query(&[("q", filter)]);
    backend::json(cx, query).await
}

pub async fn update(cx: &Cx, key: &str, text: &TextChanges<'_>) -> Result<Translation, BackendError> {
    let request = backend::request(cx, Method::PUT, &format!("/api/admin/translations/{}", segment(key))).json(text);
    backend::json(cx, request).await
}

pub async fn reset(cx: &Cx, key: &str) -> Result<Translation, BackendError> {
    let path = format!("/api/admin/translations/{}/customization", segment(key));
    backend::json(cx, backend::request(cx, Method::DELETE, &path)).await
}
