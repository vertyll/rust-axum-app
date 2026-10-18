use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

use super::ApiError;
use crate::identity::guards::CallerToken;
use crate::keycloak_account::application::{AccountApi, KeycloakAccountService};
use crate::keycloak_account::domain::SecondFactor;
use crate::shared_http::problem::JsonBody;
use crate::translations::contracts::Language;

type Account<A> = State<Arc<KeycloakAccountService<A>>>;

pub fn account_router<A: AccountApi>(account: Arc<KeycloakAccountService<A>>) -> Router {
    Router::new()
        .route("/me/second-factor", get(my_second_factor))
        .route("/me/language", put(keep_my_language))
        .with_state(account)
}

async fn my_second_factor<A: AccountApi>(
    State(account): Account<A>,
    CallerToken(token): CallerToken,
) -> Result<Json<SecondFactorResponse>, ApiError> {
    Ok(Json(account.second_factor(&token).await?.into()))
}

/// Stores the language the reader chose here on their Keycloak account, which
/// Keycloak's pages show a signed-in person in.
async fn keep_my_language<A: AccountApi>(
    State(account): Account<A>,
    CallerToken(token): CallerToken,
    JsonBody(request): JsonBody<AccountLanguageRequest>,
) -> Result<StatusCode, ApiError> {
    request.validate()?;
    let language = request
        .language
        .parse()
        .expect("a validated language is one this application has");
    account.keep_language(&token, language).await;
    Ok(StatusCode::NO_CONTENT)
}

/// `PUT /users/me/language`: a language this application has, which the realm
/// supports too, as a test checks.
#[derive(Debug, Deserialize, Validate)]
struct AccountLanguageRequest {
    #[validate(custom(function = "supported_language", message = "validation.invalid"))]
    language: String,
}

fn supported_language(language: &str) -> Result<(), ValidationError> {
    language
        .parse::<Language>()
        .map(drop)
        .map_err(|_| ValidationError::new("unsupported_language"))
}

/// Whether the caller set up a second factor, and the credential Keycloak's
/// `delete_credential` action removes.
#[derive(Debug, Serialize)]
struct SecondFactorResponse {
    enabled: bool,
    credential_id: Option<String>,
}

impl From<Option<SecondFactor>> for SecondFactorResponse {
    fn from(second_factor: Option<SecondFactor>) -> Self {
        Self {
            enabled: second_factor.is_some(),
            credential_id: second_factor.map(|factor| factor.credential_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::AccountLanguageRequest;

    /// The account takes the languages this application has, which Keycloak's
    /// pages must have too, or they would open in a language they lack.
    #[test]
    fn every_language_of_the_application_is_one_the_realm_supports() {
        let export: serde_json::Value = serde_json::from_str(include_str!("../../../../../keycloak/realm-export.json"))
            .expect("the realm export is JSON");
        let supported = export["supportedLocales"]
            .as_array()
            .expect("the realm names its locales");
        for language in crate::translations::contracts::Language::ALL {
            assert!(
                supported.iter().any(|locale| locale == language.code()),
                "the realm does not support {language}"
            );
        }
    }

    #[test]
    fn an_account_takes_only_a_language_this_application_has() {
        let request: AccountLanguageRequest = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert!(request.validate().is_ok());

        let request: AccountLanguageRequest = serde_json::from_str(r#"{"language":"de"}"#).unwrap();
        assert!(request.validate().is_err());
    }
}
