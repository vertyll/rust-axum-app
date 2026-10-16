use std::fmt;

use jiff::Timestamp;
use reqwest::Method;
use reqwest::multipart::{Form, Part};
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
    /// Where the browser downloads the file; under `/uploads`, which this
    /// application passes on to the back-end.
    pub url: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

pub struct Upload {
    pub name: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

impl fmt::Debug for Upload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Upload")
            .field("name", &self.name)
            .field("mime_type", &self.mime_type)
            .field("bytes", &self.data.len())
            .finish()
    }
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

pub async fn upload(cx: &Cx, upload: Upload) -> Result<StoredFile, BackendError> {
    let part = Part::bytes(upload.data)
        .file_name(upload.name)
        .mime_str(&upload.mime_type)
        .map_err(BackendError::Malformed)?;
    let request = backend::request(cx, Method::POST, "/api/files").multipart(Form::new().part("file", part));
    backend::json(cx, request).await
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
