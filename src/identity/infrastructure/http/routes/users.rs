use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use validator::Validate;

use super::Identity;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::UserId;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::{Auth, RequireAdmin};
use crate::identity::infrastructure::http::requests::{RegisterRequest, UpdateUserRequest};
use crate::identity::infrastructure::http::responses::UserResponse;
use crate::shared_infrastructure::problem::{JsonBody, PathParam};

pub fn users_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
	Router::new()
		.route("/", get(list_users).post(create_user))
		.route("/{id}", get(get_user).put(update_user).delete(delete_user))
		.with_state(identity)
}

async fn list_users<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(_claims): Auth,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
	let users = identity.list_users().await?;
	Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

async fn get_user<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(_claims): Auth,
	PathParam(id): PathParam<i64>,
) -> Result<Json<UserResponse>, ApiError> {
	let user = identity.get_user(UserId(id)).await?;
	Ok(Json(user.into()))
}

async fn create_user<P: IdentityPorts>(
	State(identity): Identity<P>,
	RequireAdmin(_claims): RequireAdmin,
	JsonBody(request): JsonBody<RegisterRequest>,
) -> Result<Json<UserResponse>, ApiError> {
	request.validate()?;
	let user = identity.create_user(request.into_command()?).await?;
	Ok(Json(user.into()))
}

async fn update_user<P: IdentityPorts>(
	State(identity): Identity<P>,
	RequireAdmin(_claims): RequireAdmin,
	PathParam(id): PathParam<i64>,
	JsonBody(request): JsonBody<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
	request.validate()?;
	let user = identity.update_user(UserId(id), request.into_command()?).await?;
	Ok(Json(user.into()))
}

async fn delete_user<P: IdentityPorts>(
	State(identity): Identity<P>,
	RequireAdmin(_claims): RequireAdmin,
	PathParam(id): PathParam<i64>,
) -> Result<(), ApiError> {
	identity.deactivate_user(UserId(id)).await?;
	Ok(())
}
