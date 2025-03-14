use crate::common::error::error::AppError;
use crate::users::dto::create_user::CreateUserDto;
use crate::users::dto::update_user::UpdateUserDto;
use crate::users::entities::user;
use crate::users::services::users_service::UsersService;
use axum::{
	Json, Router,
	extract::{Path, State},
	routing::get,
};
use sea_orm::DatabaseConnection;
use validator::Validate;

pub fn routes(db: DatabaseConnection) -> Router {
	let users_service = UsersService::new(db);

	Router::new()
		.route("/", get(get_all_users).post(create_user))
		.route("/{id}", get(get_user_by_id).put(update_user).delete(delete_user))
		.with_state(users_service)
}

async fn get_all_users(State(service): State<UsersService>) -> Result<Json<Vec<user::Model>>, AppError> {
	let users = service.find_all().await?;
	Ok(Json(users))
}

async fn get_user_by_id(
	State(service): State<UsersService>,
	Path(id): Path<i32>,
) -> Result<Json<user::Model>, AppError> {
	let user = service.find_by_id(id).await?;
	Ok(Json(user))
}

async fn create_user(
	State(service): State<UsersService>,
	Json(dto): Json<CreateUserDto>,
) -> Result<Json<user::Model>, AppError> {
	dto.validate()?;

	let user = service.create(dto).await?;
	Ok(Json(user))
}

async fn update_user(
	State(service): State<UsersService>,
	Path(id): Path<i32>,
	Json(dto): Json<UpdateUserDto>,
) -> Result<Json<user::Model>, AppError> {
	dto.validate()?;

	let user = service.update(id, dto).await?;
	Ok(Json(user))
}

async fn delete_user(State(service): State<UsersService>, Path(id): Path<i32>) -> Result<(), AppError> {
	service.delete(id).await?;
	Ok(())
}
