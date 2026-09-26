//! Error presentation of the files HTTP adapter: the single place mapping
//! `FilesError` and validation failures to statuses and problem documents.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::files::domain::FilesError;
use crate::shared_infrastructure::problem::{self, Problem};

#[derive(Debug)]
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
			ApiError::Validation(errors) => Problem::validation(&errors),
			ApiError::Files(err) => files_problem(err),
		}
		.into_response()
	}
}

fn files_problem(err: FilesError) -> Problem {
	use FilesError as E;

	match err {
		E::NotFound => Problem::new(StatusCode::NOT_FOUND, "files.errors.not_found"),
		E::NoFileUploaded => Problem::field("file", "files.errors.no_file"),
		E::UploadFailed => Problem::new(StatusCode::BAD_REQUEST, "files.errors.upload"),
		E::InvalidStorageType => Problem::field("storage_type", "files.validators.file.storage_type.invalid"),
		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			Problem::new(StatusCode::INTERNAL_SERVER_ERROR, problem::INTERNAL)
		}
		E::CorruptData(detail) => {
			tracing::error!("corrupt data: {detail}");
			Problem::new(StatusCode::INTERNAL_SERVER_ERROR, problem::INTERNAL)
		}
		E::StorageFailure(detail) => {
			tracing::error!("storage failure: {detail}");
			Problem::new(StatusCode::INTERNAL_SERVER_ERROR, problem::INTERNAL)
		}
	}
}
