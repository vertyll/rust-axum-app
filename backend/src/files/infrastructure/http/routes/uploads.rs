//! `/uploads`, the local disk's side of signed links: the browser sends the bytes
//! of an upload here and fetches downloads from here. The signature in the link is
//! the permission, checked on every request, so these routes need no session.

use axum::Router;
use axum::body::Body;
use axum::extract::State;
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::put;
use tokio_util::io::ReaderStream;

use crate::files::domain::FilesError;
use crate::files::infrastructure::http::error::ApiError;
use crate::files::infrastructure::storage::{DownloadParams, LocalFileStorage, UploadParams, local_disposition};
use crate::shared_http::problem::{PathParam, QueryParams};

pub fn uploads_router(storage: LocalFileStorage) -> Router {
    Router::new()
        .route("/{key}", put(store_upload).get(serve_download))
        .with_state(storage)
}

async fn store_upload(
    State(storage): State<LocalFileStorage>,
    PathParam(key): PathParam<String>,
    QueryParams(params): QueryParams<UploadParams>,
    headers: HeaderMap,
    body: Body,
) -> Result<StatusCode, ApiError> {
    storage.verify_upload(&key, &params)?;
    let content_type = headers.get(header::CONTENT_TYPE).and_then(|value| value.to_str().ok());
    if content_type != Some(params.mime_type.as_str()) {
        return Err(FilesError::InvalidLink.into());
    }
    storage.write(&key, params.size, body.into_data_stream()).await?;
    Ok(StatusCode::OK)
}

async fn serve_download(
    State(storage): State<LocalFileStorage>,
    PathParam(key): PathParam<String>,
    QueryParams(params): QueryParams<DownloadParams>,
) -> Result<Response, ApiError> {
    storage.verify_download(&key, &params)?;
    let file = storage.open(&key).await?;
    let mut response = Body::from_stream(ReaderStream::new(file)).into_response();
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, header_value(&params.mime_type)?);
    headers.insert(
        header::CONTENT_DISPOSITION,
        header_value(&local_disposition(&params.name))?,
    );
    Ok(response)
}

fn header_value(value: &str) -> Result<HeaderValue, ApiError> {
    HeaderValue::from_str(value).map_err(|_| FilesError::InvalidLink.into())
}
