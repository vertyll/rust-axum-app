//! Error presentation: the single place mapping `IdentityError` and
//! request-validation failures to statuses and translated JSON bodies,
//! shape-compatible with the previous API.
//!
//! `ApiError` is the single error type identity handlers return: it wraps
//! both module errors and request-validation failures, and is the only
//! place where `IdentityError` variants are turned into HTTP status codes
//! and translated JSON bodies. Response shapes match the previous API
//! (`{"error": …}` and `{"error": …, "details": …}` for validation).

use std::borrow::Cow;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use validator::{ValidationError, ValidationErrors};

use crate::identity::domain::IdentityError;
use crate::shared_infrastructure::i18n::translate;

#[derive(Debug)]
pub enum ApiError {
	Identity(IdentityError),
	Validation(ValidationErrors),
}

impl From<IdentityError> for ApiError {
	fn from(err: IdentityError) -> Self {
		Self::Identity(err)
	}
}

impl From<ValidationErrors> for ApiError {
	fn from(errors: ValidationErrors) -> Self {
		Self::Validation(errors)
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		match self {
			ApiError::Validation(errors) => validation_response(errors),
			ApiError::Identity(err) => identity_response(err),
		}
	}
}

fn identity_response(err: IdentityError) -> Response {
	use IdentityError as E;

	match err {
		// Business-rule violations.
		E::EmailTaken => field_error("email", "already_exists", "users.errors.user_already_exists"),
		E::UsernameTaken => field_error("username", "already_exists", "users.errors.username_already_exists"),
		E::SameEmailAsCurrent => field_error("email", "same_email", "users.errors.new_email_same_as_current"),
		E::InvalidCurrentPassword => {
			field_error("current_password", "invalid", "users.errors.invalid_current_password")
		}
		E::EmailAlreadyConfirmed => field_error("email", "already_confirmed", "auth.errors.email_already_confirmed"),
		E::InvalidEmail => field_error("email", "invalid_format", "users.validators.email.invalid_format"),
		E::UsernameTooShort => field_error("username", "too_short", "users.validators.username.too_short"),

		// Authentication problems.
		E::InvalidCredentials => authentication("auth.errors.invalid_credentials"),
		E::AccountInactive => authentication("auth.errors.account_inactive"),
		E::EmailNotConfirmed => authentication("auth.errors.email_not_confirmed"),
		E::MissingBearerToken => authentication("auth.errors.missing_token"),
		E::InvalidToken => authentication("auth.errors.invalid_token"),
		E::ExpiredToken => authentication("auth.errors.expired_token"),
		E::InvalidTokenType => authentication("auth.errors.invalid_token_type"),
		E::RefreshTokenMissing => authentication("auth.errors.missing_refresh_token"),
		E::RefreshTokenInvalid => authentication("auth.errors.invalid_refresh_token"),
		E::RefreshTokenExpired => authentication("auth.errors.expired_refresh_token"),

		// Authorization.
		E::AdminRoleRequired => {
			let message = translate("auth.errors.admin_role_required");
			let body = rust_i18n::t!(
				"errors.authorization",
				message = message,
				locale = &crate::shared_infrastructure::i18n::locale()
			)
			.to_string();
			(StatusCode::FORBIDDEN, Json(json!({ "error": body }))).into_response()
		}

		E::UserNotFound => (
			StatusCode::NOT_FOUND,
			Json(json!({ "error": translate("errors.not_found") })),
		)
			.into_response(),

		// Adapter failures: log the detail, return a generic body.
		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			internal("errors.database")
		}
		E::CorruptData(detail) => {
			tracing::error!("corrupt data: {detail}");
			internal("errors.internal")
		}
		E::MailerFailure(detail) => {
			tracing::error!("mailer failure: {detail}");
			internal("errors.internal")
		}
		E::HashingFailure | E::TokenSigningFailure => internal("errors.internal"),
	}
}

fn validation_response(errors: ValidationErrors) -> Response {
	(
		StatusCode::BAD_REQUEST,
		Json(json!({ "error": translate("errors.validation"), "details": sanitized_details(&errors) })),
	)
		.into_response()
}

fn field_error(field: &'static str, code: &'static str, message_key: &str) -> Response {
	let mut errors = ValidationErrors::new();
	let mut error = ValidationError::new(code);
	error.message = Some(Cow::Owned(translate(message_key)));
	errors.add(field, error);
	validation_response(errors)
}

fn authentication(message_key: &str) -> Response {
	let message = translate(message_key);
	let body = rust_i18n::t!(
		"errors.authentication",
		message = message,
		locale = &crate::shared_infrastructure::i18n::locale()
	)
	.to_string();
	(StatusCode::UNAUTHORIZED, Json(json!({ "error": body }))).into_response()
}

fn internal(message_key: &str) -> Response {
	(
		StatusCode::INTERNAL_SERVER_ERROR,
		Json(json!({ "error": translate(message_key) })),
	)
		.into_response()
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
