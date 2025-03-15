use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::user;
use crate::users::services::users_service::UsersService;
use axum::{
	Json, Router,
	extract::{Path, State},
	routing::get,
};
use validator::Validate;

pub fn routes(app_state: AppState) -> Router {
	let users_service = UsersService::new(app_state.db.clone());

	Router::new()
		.route("/", get(get_all_users).post(create_user))
		.route("/{:id}", get(get_user_by_id).put(update_user).delete(delete_user))
		.with_state(users_service)
}

async fn get_all_users(
	JwtAuth(_claims): JwtAuth,
	State(service): State<UsersService>,
) -> Result<Json<Vec<user::Model>>, AppError> {
	let users = service.find_all().await?;
	Ok(Json(users))
}

async fn get_user_by_id(
	JwtAuth(_claims): JwtAuth,
	State(service): State<UsersService>,
	Path(id): Path<i32>,
) -> Result<Json<user::Model>, AppError> {
	let user = service.find_by_id(id).await?;
	Ok(Json(user))
}

async fn create_user(
	JwtAuth(_claims): JwtAuth,
	State(service): State<UsersService>,
	Json(dto): Json<CreateUserDto>,
) -> Result<Json<user::Model>, AppError> {
	dto.validate()?;

	let user = service.create(dto).await?;
	Ok(Json(user))
}

async fn update_user(
	JwtAuth(_claims): JwtAuth,
	State(service): State<UsersService>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateUserDto>,
) -> Result<Json<user::Model>, AppError> {
	dto.validate()?;

	let user = service.update(id, dto).await?;
	Ok(Json(user))
}

async fn delete_user(
	JwtAuth(_claims): JwtAuth,
	State(service): State<UsersService>,
	Path(id): Path<i32>,
) -> Result<(), AppError> {
	service.delete(id).await?;
	Ok(())
}
