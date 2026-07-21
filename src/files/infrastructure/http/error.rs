//! Error presentation of the files HTTP adapter: the single place mapping
//! `FilesError` and validation failures to statuses and translated bodies.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::files::domain::FilesError;
use crate::shared_infrastructure::i18n::translate;

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
			ApiError::Validation(errors) => (
				StatusCode::BAD_REQUEST,
				Json(json!({ "error": translate("errors.validation"), "details": sanitized_details(&errors) })),
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

/// Serializes validation errors for the response body without echoing the
/// submitted input back: `validator`'s derive stores the rejected value under
/// `params.value`, which must not reach response bodies (passwords included).
fn sanitized_details(errors: &validator::ValidationErrors) -> serde_json::Value {
	let mut details = serde_json::to_value(errors).unwrap_or_else(|_| json!({}));
	strip_submitted_values(&mut details);
	details
}

fn strip_submitted_values(value: &mut serde_json::Value) {
	match value {
		serde_json::Value::Object(map) => {
			// A serialized `ValidationError` always carries a `code`; only there
			// does `params.value` mean "the submitted input".
			if map.contains_key("code")
				&& let Some(params) = map.get_mut("params").and_then(serde_json::Value::as_object_mut)
			{
				params.remove("value");
			}
			map.values_mut().for_each(strip_submitted_values);
		}
		serde_json::Value::Array(items) => items.iter_mut().for_each(strip_submitted_values),
		_ => {}
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use validator::{ValidationError, ValidationErrors};

	use super::sanitized_details;

	#[test]
	fn details_never_echo_the_submitted_value() {
		let mut error = ValidationError::new("length");
		error.message = Some(Cow::Borrowed("too short"));
		error.add_param(Cow::Borrowed("min"), &8);
		error.add_param(Cow::Borrowed("value"), &"hunter2");
		let mut errors = ValidationErrors::new();
		errors.add("password", error);

		let details = sanitized_details(&errors);
		let entry = &details["password"][0];
		assert_eq!(entry["code"], "length");
		assert_eq!(entry["message"], "too short");
		assert_eq!(entry["params"]["min"], 8);
		assert!(entry["params"].get("value").is_none());
	}
}
