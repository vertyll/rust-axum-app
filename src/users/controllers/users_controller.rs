use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::extractor::role_extractor::AdminRole;
use crate::common::error::app_error::AppError;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::users;
use crate::users::services::users_service::IUsersService;
use axum::{Extension, Json, Router, extract::Path, routing::get};
use std::sync::Arc;
use validator::Validate;

pub fn routes() -> Router {
	Router::new()
		.route("/", get(get_all_users).post(create_user))
		.route("/{:id}", get(get_user_by_id).put(update_user).delete(delete_user))
}

async fn get_all_users(
	JwtAuth(_claims): JwtAuth,
	Extension(users_service): Extension<Arc<dyn IUsersService>>,
) -> Result<Json<Vec<users::Model>>, AppError> {
	let users = users_service.find_all().await?;
	Ok(Json(users))
}

async fn get_user_by_id(
	JwtAuth(_claims): JwtAuth,
	Extension(users_service): Extension<Arc<dyn IUsersService>>,
	Path(id): Path<i32>,
) -> Result<Json<users::Model>, AppError> {
	let user = users_service.find_by_id(id).await?;
	Ok(Json(user))
}

async fn create_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(users_service): Extension<Arc<dyn IUsersService>>,
	Json(dto): Json<CreateUserDto>,
) -> Result<Json<users::Model>, AppError> {
	dto.validate()?;
	let user = users_service.create(dto).await?;
	Ok(Json(user))
}

async fn update_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(users_service): Extension<Arc<dyn IUsersService>>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateUserDto>,
) -> Result<Json<users::Model>, AppError> {
	dto.validate()?;
	let user = users_service.update(id, dto).await?;
	Ok(Json(user))
}

async fn delete_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	Extension(users_service): Extension<Arc<dyn IUsersService>>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	users_service.delete(id).await?;
	Ok(())
}
