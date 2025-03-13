use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
	ValidationError(validator::ValidationErrors),
	DatabaseError(sqlx::Error),
	ConfigError(String),
	NotFound,
	InternalError,
	AuthenticationError(String),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, error_body) = match self {
			AppError::ValidationError(errors) => (StatusCode::BAD_REQUEST, json!({ "errors": errors })),
			AppError::DatabaseError(err) => {
				tracing::error!("Database error: {:?}", err);
				(StatusCode::INTERNAL_SERVER_ERROR, json!({"error": "Database error"}))
			}
			AppError::ConfigError(msg) => {
				tracing::error!("Configuration error: {}", msg);
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					json!({"error": "Configuration error"}),
				)
			}
			AppError::NotFound => (StatusCode::NOT_FOUND, json!({"error": "Resource not found"})),
			AppError::InternalError => (
				StatusCode::INTERNAL_SERVER_ERROR,
				json!({"error": "Internal server error"}),
			),
			AppError::AuthenticationError(message) => (StatusCode::UNAUTHORIZED, json!({"error": message})),
		};

		(status, Json(error_body)).into_response()
	}
}

impl From<validator::ValidationErrors> for AppError {
	fn from(errors: validator::ValidationErrors) -> Self {
		AppError::ValidationError(errors)
	}
}

impl From<sqlx::Error> for AppError {
	fn from(err: sqlx::Error) -> Self {
		if let sqlx::Error::RowNotFound = err {
			return AppError::NotFound;
		}
		AppError::DatabaseError(err)
	}
}

impl From<config::ConfigError> for AppError {
	fn from(err: config::ConfigError) -> Self {
		AppError::ConfigError(err.to_string())
	}
}
