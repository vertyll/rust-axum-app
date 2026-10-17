use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use validator::Validate;

use super::Files;
use crate::files::application::permissions::{FilesManage, FilesView};
use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository};
use crate::files::infrastructure::http::error::ApiError;
use crate::files::infrastructure::http::requests::{UpdateFileRequest, UploadTicketRequest};
use crate::files::infrastructure::http::responses::{FileResponse, LinkResponse, UploadTicketResponse};
use crate::identity::Permitted;
use crate::shared_http::pagination::{PageQuery, PageResponse};
use crate::shared_http::problem::{JsonBody, PathParam, QueryParams};

pub fn files_router<R, S>(files: Arc<FilesService<R, S>>) -> Router
where
    R: FileRepository,
    S: FileStorage,
{
    Router::new()
        .route("/", get(list_files))
        .route("/upload-ticket", post(upload_ticket))
        .route("/{id}", get(get_file).put(update_file).delete(delete_file))
        .route("/{id}/confirm", post(confirm_upload))
        .route("/{id}/download-link", get(download_link))
        .route("/{id}/soft-delete", post(soft_delete_file))
        .with_state(files)
}

async fn list_files<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesView>,
    QueryParams(query): QueryParams<PageQuery>,
) -> Result<Json<PageResponse<FileResponse>>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    query.validate()?;
    Ok(Json(files.list(query.request()).await?.into()))
}

async fn get_file<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesView>,
    PathParam(id): PathParam<i64>,
) -> Result<Json<FileResponse>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    Ok(Json(files.get(FileId(id)).await?.into()))
}

async fn upload_ticket<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesManage>,
    JsonBody(request): JsonBody<UploadTicketRequest>,
) -> Result<Json<UploadTicketResponse>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    request.validate()?;
    Ok(Json(files.request_upload(request.into()).await?.into()))
}

async fn confirm_upload<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesManage>,
    PathParam(id): PathParam<i64>,
) -> Result<Json<FileResponse>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    Ok(Json(files.confirm_upload(FileId(id)).await?.into()))
}

async fn download_link<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesView>,
    PathParam(id): PathParam<i64>,
) -> Result<Json<LinkResponse>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    Ok(Json(files.download_link(FileId(id)).await?.into()))
}

async fn update_file<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesManage>,
    PathParam(id): PathParam<i64>,
    JsonBody(request): JsonBody<UpdateFileRequest>,
) -> Result<Json<FileResponse>, ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    request.validate()?;
    Ok(Json(files.update(FileId(id), request.into()).await?.into()))
}

async fn delete_file<R, S>(
    State(files): Files<R, S>,
    Permitted(_caller, _): Permitted<FilesManage>,
    PathParam(id): PathParam<i64>,
) -> Result<(), ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    files.delete(FileId(id)).await?;
    Ok(())
}

async fn soft_delete_file<R, S>(
    State(files): Files<R, S>,
    Permitted(caller, _): Permitted<FilesManage>,
    PathParam(id): PathParam<i64>,
) -> Result<(), ApiError>
where
    R: FileRepository,
    S: FileStorage,
{
    files.soft_delete(FileId(id), caller.user_id.0).await?;
    Ok(())
}
