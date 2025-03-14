use crate::auth::dto::login_dto::LoginDto;
use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::services::auth_service::{AuthResponse, AuthService};
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use axum::{Json, Router, extract::State, routing::post};
use validator::Validate;

pub fn routes(app_state: AppState, jwt_access_token_expires_in: i64) -> Router {
	let auth_service = AuthService::new(app_state, jwt_access_token_expires_in);

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
