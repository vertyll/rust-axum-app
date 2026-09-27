use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use validator::Validate;

use super::Files;
use crate::files::application::commands::UploadFile;
use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError, StorageKind};
use crate::files::infrastructure::http::error::ApiError;
use crate::files::infrastructure::http::requests::{UpdateFileRequest, UploadQuery};
use crate::files::infrastructure::http::responses::FileResponse;
use crate::identity::{Auth, RequireAdmin};
use crate::shared_infrastructure::problem::{JsonBody, PathParam, QueryParams};

pub fn files_router<R, S>(files: Arc<FilesService<R, S>>) -> Router
where
	R: FileRepository,
	S: FileStorage,
{
	Router::new()
		.route("/", get(list_files).post(upload_file))
		.route("/{id}", get(get_file).put(update_file).delete(delete_file))
		.route("/{id}/soft-delete", post(soft_delete_file))
		.with_state(files)
}

async fn list_files<R, S>(State(files): Files<R, S>, Auth(_claims): Auth) -> Result<Json<Vec<FileResponse>>, ApiError>
where
	R: FileRepository,
	S: FileStorage,
{
	let list = files.list().await?;
	Ok(Json(list.into_iter().map(FileResponse::from).collect()))
}

async fn get_file<R, S>(
	State(files): Files<R, S>,
	Auth(_claims): Auth,
	PathParam(id): PathParam<i64>,
) -> Result<Json<FileResponse>, ApiError>
where
	R: FileRepository,
	S: FileStorage,
{
	Ok(Json(files.get(FileId(id)).await?.into()))
}

async fn upload_file<R, S>(
	State(files): Files<R, S>,
	Auth(_claims): Auth,
	QueryParams(query): QueryParams<UploadQuery>,
	mut multipart: Multipart,
) -> Result<Json<FileResponse>, ApiError>
where
	R: FileRepository,
	S: FileStorage,
{
	let storage = match query.storage_type.as_deref() {
		None => StorageKind::Local,
		Some(value) => value.parse()?,
	};

	let mut upload: Option<UploadFile> = None;
	while let Some(field) = multipart.next_field().await.map_err(|err| {
		tracing::error!("error reading multipart field: {err}");
		FilesError::UploadFailed
	})? {
		if field.name() == Some("file") {
			let original_name = field.file_name().ok_or(FilesError::NoFileUploaded)?.to_string();
			let mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
			let data = field
				.bytes()
				.await
				.map_err(|err| {
					tracing::error!("error reading file data: {err}");
					FilesError::UploadFailed
				})?
				.to_vec();

			upload = Some(UploadFile {
				data,
				original_name,
				mime_type,
				storage,
			});
		}
	}

	let upload = upload.ok_or(FilesError::NoFileUploaded)?;
	Ok(Json(files.upload(upload).await?.into()))
}

async fn update_file<R, S>(
	State(files): Files<R, S>,
	RequireAdmin(_claims): RequireAdmin,
	PathParam(id): PathParam<i64>,
	JsonBody(request): JsonBody<UpdateFileRequest>,
) -> Result<Json<FileResponse>, ApiError>
where
	R: FileRepository,
	S: FileStorage,
{
	request.validate()?;
	Ok(Json(files.update(FileId(id), request.into_command()?).await?.into()))
}

async fn delete_file<R, S>(
	State(files): Files<R, S>,
	RequireAdmin(_claims): RequireAdmin,
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
	RequireAdmin(claims): RequireAdmin,
	PathParam(id): PathParam<i64>,
) -> Result<(), ApiError>
where
	R: FileRepository,
	S: FileStorage,
{
	files.soft_delete(FileId(id), claims.sub).await?;
	Ok(())
}
