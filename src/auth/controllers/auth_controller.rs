use crate::auth::dto::login::LoginDto;
use crate::auth::dto::register::RegisterDto;
use crate::auth::services::auth_service::{AuthResponse, AuthService};
use crate::common::error::error::AppError;
use axum::{Json, Router, extract::State, routing::post};
use sea_orm::DatabaseConnection;
use validator::Validate;

pub fn routes(db: DatabaseConnection, jwt_access_token_secret: String, jwt_access_token_expires_in: i64) -> Router {
	let auth_service = AuthService::new(db, jwt_access_token_secret, jwt_access_token_expires_in);

	Router::new()
		.route("/register", post(register))
		.route("/login", post(login))
		.with_state(auth_service)
}

async fn register(
	State(service): State<AuthService>,
	Json(dto): Json<RegisterDto>,
) -> Result<Json<AuthResponse>, AppError> {
	dto.validate()?;

	let response = service.register(dto).await?;
	Ok(Json(response))
}

async fn login(State(service): State<AuthService>, Json(dto): Json<LoginDto>) -> Result<Json<AuthResponse>, AppError> {
	let response = service.login(dto).await?;
	Ok(Json(response))
}
