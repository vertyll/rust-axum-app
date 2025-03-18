use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::extractor::role_extractor::AdminRole;
use crate::common::error::app_error::AppError;
use crate::files::dto::update_file_dto::UpdateFileDto;
use crate::files::entities::files;
use crate::files::services::files_service::IFilesService;
use axum::{
	Extension, Json, Router,
	extract::{Multipart, Path, Query},
	routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
	storage_type: Option<String>,
}

pub fn routes() -> Router {
	Router::new()
		.route("/", get(get_all_files).post(upload_file))
		.route("/{:id}", get(get_file_by_id).put(update_file).delete(delete_file))
		.route("/{:id}/soft-delete", post(soft_delete_file))
}

async fn get_all_files(
	JwtAuth(_claims): JwtAuth,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
) -> Result<Json<Vec<files::Model>>, AppError> {
	let files = files_service.find_all().await?;
	Ok(Json(files))
}

async fn get_file_by_id(
	JwtAuth(_claims): JwtAuth,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
	Path(id): Path<i32>,
) -> Result<Json<files::Model>, AppError> {
	let file = files_service.find_by_id(id).await?;
	Ok(Json(file))
}

async fn upload_file(
	JwtAuth(_claims): JwtAuth,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
	Query(query): Query<UploadQuery>,
	multipart: Multipart,
) -> Result<Json<files::Model>, AppError> {
	let file = files_service.upload(multipart, query.storage_type).await?;
	Ok(Json(file))
}

async fn update_file(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateFileDto>,
) -> Result<Json<files::Model>, AppError> {
	dto.validate()?;

	let file = files_service.update(id, dto).await?;
	Ok(Json(file))
}

async fn delete_file(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	files_service.delete(id).await?;
	Ok(())
}

async fn soft_delete_file(
	JwtAuth(claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(files_service): Extension<Arc<dyn IFilesService>>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	files_service.soft_delete(id, claims.sub).await?;
	Ok(())
}
