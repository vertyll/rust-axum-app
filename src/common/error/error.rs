use crate::i18n::setup::translate;
use axum::{
	Json,
	http::StatusCode,
	response::{IntoResponse, Response},
};
use sea_orm::DbErr;
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
	ValidationError(validator::ValidationErrors),
	DatabaseError(String),
	ConfigError(String),
	NotFound,
	InternalError,
	AuthenticationError(String),
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let (status, error_body) = match self {
			AppError::ValidationError(errors) => (
				StatusCode::BAD_REQUEST,
				json!({ "error": translate("errors.validation"), "details": errors }),
			),
			AppError::DatabaseError(err) => {
				tracing::error!("Database error: {}", err);
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					json!({"error": translate("errors.database")}),
				)
			}
			AppError::ConfigError(msg) => {
				tracing::error!("Configuration error: {}", msg);
				(
					StatusCode::INTERNAL_SERVER_ERROR,
					json!({"error": translate("errors.config")}),
				)
			}
			AppError::NotFound => (StatusCode::NOT_FOUND, json!({"error": translate("errors.not_found")})),
			AppError::InternalError => (
				StatusCode::INTERNAL_SERVER_ERROR,
				json!({"error": translate("errors.internal")}),
			),
			AppError::AuthenticationError(message) => {
				let translated = rust_i18n::t!("errors.authentication", message = message);
				(StatusCode::UNAUTHORIZED, json!({"error": translated}))
			}
		};

		(status, Json(error_body)).into_response()
	}
}

impl From<validator::ValidationErrors> for AppError {
	fn from(errors: validator::ValidationErrors) -> Self {
		AppError::ValidationError(errors)
	}
}

impl From<DbErr> for AppError {
	fn from(err: DbErr) -> Self {
		match err {
			DbErr::RecordNotFound(_) => AppError::NotFound,
			_ => AppError::DatabaseError(err.to_string()),
		}
	}
}

impl From<config::ConfigError> for AppError {
	fn from(err: config::ConfigError) -> Self {
		AppError::ConfigError(err.to_string())
	}
}
