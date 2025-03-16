use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::extractor::role_extractor::AdminRole;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::users;
use crate::users::repositories::users_repository::UsersRepositoryTrait;
use crate::users::services::users_service::{UsersService, UsersServiceTrait};
use axum::{
	Json, Router,
	extract::{Path, State},
	routing::get,
};
use std::sync::Arc;
use validator::Validate;

#[derive(Clone)]
pub struct UsersControllerStateDyn {
	users_service: Arc<dyn UsersServiceTrait>,
}

pub fn routes(app_state: AppState) -> Router {
	let users_service = Arc::new(UsersService::new(app_state.db.clone()));

	let dependencies_state = UsersControllerStateDyn {
		users_service: users_service.clone(),
	};

	Router::new()
		.route("/", get(get_all_users).post(create_user))
		.route("/{:id}", get(get_user_by_id).put(update_user).delete(delete_user))
		.with_state(dependencies_state)
}

async fn get_all_users(
	JwtAuth(_claims): JwtAuth,
	State(dependencies): State<UsersControllerStateDyn>,
) -> Result<Json<Vec<users::Model>>, AppError> {
	let users = dependencies.users_service.find_all().await?;
	Ok(Json(users))
}

async fn get_user_by_id(
	JwtAuth(_claims): JwtAuth,
	State(dependencies): State<UsersControllerStateDyn>,
	Path(id): Path<i32>,
) -> Result<Json<users::Model>, AppError> {
	let user = dependencies.users_service.find_by_id(id).await?;
	Ok(Json(user))
}

async fn create_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<UsersControllerStateDyn>,
	Json(dto): Json<CreateUserDto>,
) -> Result<Json<users::Model>, AppError> {
	dto.validate()?;

	let user = dependencies.users_service.create(dto).await?;
	Ok(Json(user))
}

async fn update_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<UsersControllerStateDyn>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateUserDto>,
) -> Result<Json<users::Model>, AppError> {
	dto.validate()?;

	let user = dependencies.users_service.update(id, dto).await?;
	Ok(Json(user))
}

async fn delete_user(
	JwtAuth(_claims): JwtAuth,
	_admin_role: AdminRole,
	State(dependencies): State<UsersControllerStateDyn>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	dependencies.users_service.delete(id).await?;
	Ok(())
}
