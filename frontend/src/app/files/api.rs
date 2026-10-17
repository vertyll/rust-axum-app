use jiff::Timestamp;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

use crate::backend::{self, BackendError, Page, PageRequest};

#[derive(Debug, Clone, Deserialize)]
pub struct StoredFile {
    pub id: i64,
    pub original_name: String,
    pub mime_type: String,
    pub encoding: String,
    pub size: i64,
    pub storage_type: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Where the browser fetches a file's bytes, for a few minutes.
#[derive(Debug, Clone, Deserialize)]
pub struct Link {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct FileChanges<'a> {
    pub original_name: &'a str,
    pub mime_type: &'a str,
    pub encoding: &'a str,
}

pub async fn list(cx: &Cx, request: PageRequest) -> Result<Page<StoredFile>, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/files").query(&request)).await
}

pub async fn get(cx: &Cx, id: i64) -> Result<StoredFile, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, &format!("/api/files/{id}"))).await
}

pub async fn download_link(cx: &Cx, id: i64) -> Result<Link, BackendError> {
    backend::json(
        cx,
        backend::request(cx, Method::GET, &format!("/api/files/{id}/download-link")),
    )
    .await
}

pub async fn update(cx: &Cx, id: i64, changes: &FileChanges<'_>) -> Result<StoredFile, BackendError> {
    let request = backend::request(cx, Method::PUT, &format!("/api/files/{id}")).json(changes);
    backend::json(cx, request).await
}

pub async fn soft_delete(cx: &Cx, id: i64) -> Result<(), BackendError> {
    let request = backend::request(cx, Method::POST, &format!("/api/files/{id}/soft-delete"));
    backend::send(cx, request).await?;
    Ok(())
}

pub async fn delete(cx: &Cx, id: i64) -> Result<(), BackendError> {
    backend::send(cx, backend::request(cx, Method::DELETE, &format!("/api/files/{id}"))).await?;
    Ok(())
}
