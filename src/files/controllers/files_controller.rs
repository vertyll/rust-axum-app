use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::extractor::role_extractor::AdminRole;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::files::dto::update_file_dto::UpdateFileDto;
use crate::files::entities::files;
use crate::files::services::files_service::{FilesService, FilesServiceTrait};
use axum::{
	Json, Router,
	extract::{Multipart, Path, Query, State},
	routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

#[derive(Clone)]
pub struct FilesControllerStateDyn {
	files_service: Arc<dyn FilesServiceTrait>,
}

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
	storage_type: Option<String>,
}

pub fn routes(app_state: AppState) -> Router {
	let files_service = Arc::new(FilesService::new(app_state.db.clone(), app_state.config.clone()));

	let dependencies_state = FilesControllerStateDyn {
		files_service: files_service.clone(),
	};

	Router::new()
		.route("/", get(get_all_files).post(upload_file))
		.route("/{:id}", get(get_file_by_id).put(update_file).delete(delete_file))
		.route("/{:id}/soft-delete", post(soft_delete_file))
		.with_state(dependencies_state)
}

async fn get_all_files(
	JwtAuth(_claims): JwtAuth,
	State(dependencies): State<FilesControllerStateDyn>,
) -> Result<Json<Vec<files::Model>>, AppError> {
	let files = dependencies.files_service.find_all().await?;
	Ok(Json(files))
}

async fn get_file_by_id(
	JwtAuth(_claims): JwtAuth,
	State(dependencies): State<FilesControllerStateDyn>,
	Path(id): Path<i32>,
) -> Result<Json<files::Model>, AppError> {
	let file = dependencies.files_service.find_by_id(id).await?;
	Ok(Json(file))
}

async fn upload_file(
	JwtAuth(_claims): JwtAuth,
	State(dependencies): State<FilesControllerStateDyn>,
	Query(query): Query<UploadQuery>,
	multipart: Multipart,
) -> Result<Json<files::Model>, AppError> {
	let file = dependencies.files_service.upload(multipart, query.storage_type).await?;
	Ok(Json(file))
}

async fn update_file(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<FilesControllerStateDyn>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateFileDto>,
) -> Result<Json<files::Model>, AppError> {
	dto.validate()?;

	let file = dependencies.files_service.update(id, dto).await?;
	Ok(Json(file))
}

async fn delete_file(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<FilesControllerStateDyn>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	dependencies.files_service.delete(id).await?;
	Ok(())
}

async fn soft_delete_file(
	JwtAuth(claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<FilesControllerStateDyn>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	dependencies.files_service.soft_delete(id, claims.sub).await?;
	Ok(())
}
