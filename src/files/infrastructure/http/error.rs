//! Error presentation of the files HTTP adapter: the single place mapping
//! `FilesError` and validation failures to statuses and translated bodies.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::files::domain::FilesError;
use crate::shared_infrastructure::i18n::translate;

pub enum ApiError {
	Files(FilesError),
	Validation(validator::ValidationErrors),
}

impl From<FilesError> for ApiError {
	fn from(err: FilesError) -> Self {
		Self::Files(err)
	}
}

impl From<validator::ValidationErrors> for ApiError {
	fn from(errors: validator::ValidationErrors) -> Self {
		Self::Validation(errors)
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		match self {
			ApiError::Validation(errors) => (
				StatusCode::BAD_REQUEST,
				Json(json!({ "error": translate("errors.validation"), "details": errors })),
			)
				.into_response(),
			ApiError::Files(err) => files_response(err),
		}
	}
}

fn files_response(err: FilesError) -> Response {
	use FilesError as E;

	let (status, message) = match err {
		E::NotFound => (StatusCode::NOT_FOUND, translate("errors.not_found")),
		E::NoFileUploaded => (StatusCode::BAD_REQUEST, translate("files.errors.no_file")),
		E::UploadFailed => (StatusCode::BAD_REQUEST, translate("files.errors.upload")),
		E::InvalidStorageType => (
			StatusCode::BAD_REQUEST,
			translate("files.validators.file.storage_type.invalid"),
		),
		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			(StatusCode::INTERNAL_SERVER_ERROR, translate("errors.database"))
		}
		E::CorruptData(detail) => {
			tracing::error!("corrupt data: {detail}");
			(StatusCode::INTERNAL_SERVER_ERROR, translate("errors.internal"))
		}
		E::StorageFailure(detail) => {
			tracing::error!("storage failure: {detail}");
			(StatusCode::INTERNAL_SERVER_ERROR, translate("errors.internal"))
		}
	};

	(status, Json(json!({ "error": message }))).into_response()
}
