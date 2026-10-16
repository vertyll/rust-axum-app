//! Error presentation: the single place mapping `IdentityError` to statuses
//! and problem documents.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::identity::domain::IdentityError;
use crate::shared_http::problem::{self, Problem};

#[derive(Debug)]
pub enum ApiError {
    Identity(IdentityError),
    Validation(validator::ValidationErrors),
}

impl From<IdentityError> for ApiError {
    fn from(err: IdentityError) -> Self {
        Self::Identity(err)
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
            ApiError::Identity(err) => identity_problem(err),
            ApiError::Validation(errors) => Problem::validation(&errors),
        }
        .into_response()
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
        E::PermissionDenied(permission) => {
            Problem::new(StatusCode::FORBIDDEN, "auth.errors.permission_denied").with_arg("permission", permission)
        }
        E::UserNotFound => Problem::new(StatusCode::NOT_FOUND, "auth.errors.user_not_found"),
        E::InvalidRoleName => Problem::field("name", "roles.validators.name.invalid_format"),
        E::RoleNotFound => Problem::new(StatusCode::NOT_FOUND, "roles.errors.not_found"),
        E::RoleAlreadyExists => Problem::new(StatusCode::CONFLICT, "roles.errors.already_exists"),
        E::UnknownPermission(name) => Problem::field("permissions", "roles.validators.permissions.unknown")
            .with_field_arg("permissions", "permission", name),
        E::LastUnrestrictedRole => Problem::new(StatusCode::CONFLICT, "roles.errors.last_unrestricted_role"),
        E::SystemRole => Problem::new(StatusCode::CONFLICT, "roles.errors.system_role"),
        E::UnrestrictedRole => Problem::new(StatusCode::CONFLICT, "roles.errors.unrestricted_role"),
        E::RoleStillAssigned => Problem::new(StatusCode::CONFLICT, "roles.errors.still_assigned"),
        E::LastUnrestrictedHolder => Problem::new(StatusCode::CONFLICT, "roles.errors.last_unrestricted_holder"),
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
            (IdentityError::PermissionDenied("USERS_VIEW"), StatusCode::FORBIDDEN),
            (IdentityError::RoleAlreadyExists, StatusCode::CONFLICT),
            (IdentityError::LastUnrestrictedRole, StatusCode::CONFLICT),
            (IdentityError::InvalidRoleName, StatusCode::BAD_REQUEST),
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
            assert_eq!(ApiError::from(err).into_response().status(), status);
        }
    }
}
