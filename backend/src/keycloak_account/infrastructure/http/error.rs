//! Error presentation of the Keycloak account adapter: the single place
//! mapping `KeycloakAccountError` and validation failures to problem documents.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::keycloak_account::domain::KeycloakAccountError;
use crate::shared_http::problem::Problem;

#[derive(Debug)]
pub enum ApiError {
    Account(KeycloakAccountError),
    Validation(validator::ValidationErrors),
}

impl From<KeycloakAccountError> for ApiError {
    fn from(err: KeycloakAccountError) -> Self {
        Self::Account(err)
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
            ApiError::Account(KeycloakAccountError::Unavailable(detail)) => {
                tracing::error!("Keycloak's account API is unavailable: {detail}");
                Problem::new(StatusCode::SERVICE_UNAVAILABLE, "keycloak_account.errors.unavailable")
            }
        }
        .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::response::IntoResponse;

    use super::ApiError;
    use crate::keycloak_account::domain::KeycloakAccountError;

    #[test]
    fn an_unavailable_keycloak_is_a_service_unavailable() {
        let response = ApiError::from(KeycloakAccountError::Unavailable("down".into())).into_response();

        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    }
}
