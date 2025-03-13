use axum::{Router, routing::post, Json, extract::State};
use sqlx::PgPool;
use validator::Validate;
use crate::auth::dto::login::LoginDto;
use crate::auth::dto::register::RegisterDto;
use crate::auth::services::auth_service::{AuthResponse, AuthService};
use crate::common::error::AppError;

pub fn routes(db_pool: PgPool, jwt_secret: String) -> Router {
    let auth_service = AuthService::new(db_pool, jwt_secret);

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

async fn login(
    State(service): State<AuthService>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<AuthResponse>, AppError> {
    let response = service.login(dto).await?;
    Ok(Json(response))
}
