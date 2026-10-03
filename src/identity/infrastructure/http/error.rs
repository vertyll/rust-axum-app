//! Error presentation: the single place mapping `IdentityError` to statuses
//! and problem documents.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::identity::domain::IdentityError;
use crate::shared_infrastructure::problem::{self, Problem};

#[derive(Debug)]
pub struct ApiError(pub IdentityError);

impl From<IdentityError> for ApiError {
	fn from(err: IdentityError) -> Self {
		Self(err)
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		identity_problem(self.0).into_response()
	}
}

fn identity_problem(err: IdentityError) -> Problem {
	use IdentityError as E;
	match err {
		E::InvalidEmail => Problem::field("email", "users.validators.email.invalid_format"),
		E::Unauthenticated => unauthorized("auth.errors.authentication_required"),
		E::InvalidToken => unauthorized("auth.errors.invalid_token"),
		E::SignInRejected => unauthorized("auth.errors.sign_in_rejected"),
		E::SessionExpired => unauthorized("auth.errors.session_expired"),
		E::AdminRoleRequired => Problem::new(StatusCode::FORBIDDEN, "auth.errors.admin_role_required"),
		E::UserNotFound => Problem::new(StatusCode::NOT_FOUND, "auth.errors.user_not_found"),
		E::IdentityProviderUnavailable(detail) => {
			tracing::error!("identity provider unavailable: {detail}");
			Problem::new(
				StatusCode::SERVICE_UNAVAILABLE,
				"auth.errors.identity_provider_unavailable",
			)
		}
		E::SessionStoreFailure(detail) => {
			tracing::error!("session store failure: {detail}");
			internal()
		}
		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			internal()
		}
		E::CorruptData(detail) => {
			tracing::error!("corrupt data: {detail}");
			internal()
		}
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
	use axum::http::StatusCode;
	use axum::response::IntoResponse;

	use super::ApiError;
	use crate::identity::domain::IdentityError;

	#[test]
	fn maps_errors_to_statuses() {
		let cases = [
			(IdentityError::Unauthenticated, StatusCode::UNAUTHORIZED),
			(IdentityError::SessionExpired, StatusCode::UNAUTHORIZED),
			(IdentityError::AdminRoleRequired, StatusCode::FORBIDDEN),
			(IdentityError::UserNotFound, StatusCode::NOT_FOUND),
			(
				IdentityError::IdentityProviderUnavailable("down".into()),
				StatusCode::SERVICE_UNAVAILABLE,
			),
			(
				IdentityError::PersistenceFailure("down".into()),
				StatusCode::INTERNAL_SERVER_ERROR,
			),
		];
		for (err, status) in cases {
			assert_eq!(ApiError(err).into_response().status(), status);
		}
	}
}
