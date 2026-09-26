//! Error presentation: the single place mapping `IdentityError` and
//! request-validation failures to statuses and problem documents.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use validator::ValidationErrors;

use crate::identity::domain::IdentityError;
use crate::shared_infrastructure::problem::{self, Problem};

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
			ApiError::Validation(errors) => Problem::validation(&errors),
			ApiError::Identity(err) => identity_problem(err),
		}
		.into_response()
	}
}

fn identity_problem(err: IdentityError) -> Problem {
	use IdentityError as E;

	match err {
		E::EmailTaken => Problem::field("email", "users.errors.user_already_exists"),
		E::UsernameTaken => Problem::field("username", "users.errors.username_already_exists"),
		E::SameEmailAsCurrent => Problem::field("email", "users.errors.new_email_same_as_current"),
		E::InvalidCurrentPassword => Problem::field("current_password", "users.errors.invalid_current_password"),
		E::EmailAlreadyConfirmed => Problem::field("email", "auth.errors.email_already_confirmed"),
		E::InvalidEmail => Problem::field("email", "users.validators.email.invalid_format"),
		E::UsernameTooShort => Problem::field("username", "users.validators.username.too_short").with_field_arg(
			"username",
			"min",
			crate::identity::domain::Username::MIN_LENGTH,
		),

		E::InvalidCredentials => unauthorized("auth.errors.invalid_credentials"),
		E::AccountInactive => unauthorized("auth.errors.account_inactive"),
		E::EmailNotConfirmed => unauthorized("auth.errors.email_not_confirmed"),
		E::MissingBearerToken => unauthorized("auth.errors.missing_token"),
		E::InvalidToken => unauthorized("auth.errors.invalid_token"),
		E::ExpiredToken => unauthorized("auth.errors.expired_token"),
		E::InvalidTokenType => unauthorized("auth.errors.invalid_token_type"),
		E::RefreshTokenMissing => unauthorized("auth.errors.missing_refresh_token"),
		E::RefreshTokenInvalid => unauthorized("auth.errors.invalid_refresh_token"),
		E::RefreshTokenExpired => unauthorized("auth.errors.expired_refresh_token"),

		E::AdminRoleRequired => Problem::new(StatusCode::FORBIDDEN, "auth.errors.admin_role_required"),
		E::UserNotFound => Problem::new(StatusCode::NOT_FOUND, "auth.errors.user_not_found"),

		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			internal()
		}
		E::CorruptData(detail) => {
			tracing::error!("corrupt data: {detail}");
			internal()
		}
		E::MailerFailure(detail) => {
			tracing::error!("mailer failure: {detail}");
			internal()
		}
		E::HashingFailure | E::TokenSigningFailure => internal(),
	}
}

fn unauthorized(key: &str) -> Problem {
	Problem::new(StatusCode::UNAUTHORIZED, key)
}

fn internal() -> Problem {
	Problem::new(StatusCode::INTERNAL_SERVER_ERROR, problem::INTERNAL)
}

#[cfg(test)]
mod tests {
	use axum::response::IntoResponse;

	use super::ApiError;
	use crate::identity::domain::IdentityError;

	#[test]
	fn business_rule_is_a_field_problem() {
		let response = ApiError::from(IdentityError::EmailTaken).into_response();

		assert_eq!(response.status(), 400);
		assert_eq!(response.headers()["content-type"], "application/problem+json");
	}

	#[test]
	fn adapter_failure_hides_its_detail() {
		let response = ApiError::from(IdentityError::PersistenceFailure("secret".into())).into_response();

		assert_eq!(response.status(), 500);
	}
}
