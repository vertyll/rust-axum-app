//! Keycloak's account API, called with the person's own access token.

use std::time::Duration;

use reqwest::header::ACCEPT;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::keycloak_account::application::AccountApi;
use crate::keycloak_account::domain::{KeycloakAccountError, SecondFactor};
use crate::translations::contracts::Language;

const TIMEOUT: Duration = Duration::from_secs(10);

/// The credential type Keycloak's TOTP and HOTP setup stores.
const OTP_CREDENTIAL: &str = "otp";

const LOCALE_ATTRIBUTE: &str = "locale";

#[derive(Clone)]
pub struct HttpAccountApi {
    http: reqwest::Client,
    account_url: String,
}

impl HttpAccountApi {
    /// The account API of the realm at `realm_url`, as this application reaches it.
    pub fn new(realm_url: &str) -> Result<Self, KeycloakAccountError> {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(unavailable)?;
        Ok(Self {
            http,
            account_url: format!("{}/account/", realm_url.trim_end_matches('/')),
        })
    }
}

impl AccountApi for HttpAccountApi {
    async fn second_factor(&self, access_token: &str) -> Result<Option<SecondFactor>, KeycloakAccountError> {
        let response = self
            .http
            .get(format!("{}credentials?type={OTP_CREDENTIAL}", self.account_url))
            .bearer_auth(access_token)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(unavailable)?;
        let credentials: Vec<CredentialType> = checked(response)?.json().await.map_err(unavailable)?;
        Ok(credentials
            .into_iter()
            .filter(|credential| credential.kind == OTP_CREDENTIAL)
            .flat_map(|credential| credential.user_credential_metadatas)
            .map(|metadata| SecondFactor {
                credential_id: metadata.credential.id,
            })
            .next())
    }

    async fn set_language(&self, access_token: &str, language: Language) -> Result<(), KeycloakAccountError> {
        let response = self
            .http
            .get(&self.account_url)
            .bearer_auth(access_token)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .map_err(unavailable)?;
        let mut account: Value = checked(response)?.json().await.map_err(unavailable)?;
        if account["attributes"][LOCALE_ATTRIBUTE][0] == language.code() {
            return Ok(());
        }
        // Keycloak replaces the account with what is sent, so it goes back whole, with only its language changed.
        if !account["attributes"].is_object() {
            account["attributes"] = json!({});
        }
        account["attributes"][LOCALE_ATTRIBUTE] = json!([language.code()]);
        let response = self
            .http
            .post(&self.account_url)
            .bearer_auth(access_token)
            .json(&account)
            .send()
            .await
            .map_err(unavailable)?;
        checked(response).map(drop)
    }
}

/// One credential type in Keycloak's account API, with the person's credentials of it.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialType {
    #[serde(rename = "type")]
    kind: String,
    #[serde(default)]
    user_credential_metadatas: Vec<CredentialMetadata>,
}

#[derive(Deserialize)]
struct CredentialMetadata {
    credential: Credential,
}

#[derive(Deserialize)]
struct Credential {
    id: String,
}

fn checked(response: reqwest::Response) -> Result<reqwest::Response, KeycloakAccountError> {
    if response.status().is_success() {
        Ok(response)
    } else {
        Err(KeycloakAccountError::Unavailable(response.status().to_string()))
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "passed to `map_err`, which hands over the error it maps"
)]
fn unavailable(err: reqwest::Error) -> KeycloakAccountError {
    KeycloakAccountError::Unavailable(err.to_string())
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use axum::http::HeaderMap;
    use axum::http::header::AUTHORIZATION;
    use axum::routing::get;
    use axum::{Json, Router};
    use serde_json::{Value, json};

    use super::HttpAccountApi;
    use crate::keycloak_account::application::AccountApi;
    use crate::translations::contracts::Language;

    async fn account_api(app: Router) -> HttpAccountApi {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        HttpAccountApi::new(&format!("http://{address}/realms/test")).unwrap()
    }

    #[tokio::test]
    async fn the_second_factor_is_read_from_the_account_api_with_the_callers_token() {
        let app = Router::new().route(
            "/realms/test/account/credentials",
            get(|headers: HeaderMap| async move {
                assert_eq!(headers[AUTHORIZATION], "Bearer access-1");
                r#"[{"type":"otp","category":"two-factor","userCredentialMetadatas":[{"credential":{"id":"otp-1","type":"otp","userLabel":"phone"}}]}]"#
            }),
        );
        let api = account_api(app).await;

        let factor = api.second_factor("access-1").await.unwrap();

        assert_eq!(factor.map(|factor| factor.credential_id).as_deref(), Some("otp-1"));
    }

    #[tokio::test]
    async fn no_second_factor_is_none() {
        let app = Router::new().route(
            "/realms/test/account/credentials",
            get(|| async { r#"[{"type":"otp","category":"two-factor","userCredentialMetadatas":[]}]"# }),
        );
        let api = account_api(app).await;

        assert_eq!(api.second_factor("access-1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn the_language_is_written_back_with_the_rest_of_the_account() {
        let written: Arc<Mutex<Option<(String, Value)>>> = Arc::default();
        let sink = written.clone();
        let app = Router::new().route(
            "/realms/test/account/",
            get(|| async { Json(json!({"username": "ada", "attributes": {"locale": ["en"], "nickname": ["a"]}})) })
                .post(move |headers: HeaderMap, Json(account): Json<Value>| {
                    let sink = sink.clone();
                    async move {
                        let token = headers[AUTHORIZATION].to_str().unwrap().to_owned();
                        *sink.lock().unwrap() = Some((token, account));
                    }
                }),
        );
        let api = account_api(app).await;

        api.set_language("access-1", Language::Pl).await.unwrap();

        let (token, account) = written.lock().unwrap().clone().unwrap();
        assert_eq!(token, "Bearer access-1");
        assert_eq!(
            account,
            json!({"username": "ada", "attributes": {"locale": ["pl"], "nickname": ["a"]}})
        );
    }

    #[tokio::test]
    async fn a_language_the_account_already_has_is_not_written() {
        let app = Router::new().route(
            "/realms/test/account/",
            // A write would be refused, and fail the call.
            get(|| async { Json(json!({"attributes": {"locale": ["pl"]}})) })
                .post(|| async { axum::http::StatusCode::CONFLICT }),
        );
        let api = account_api(app).await;

        api.set_language("access-1", Language::Pl).await.unwrap();
    }

    #[tokio::test]
    async fn a_refusal_is_reported_as_keycloak_being_unavailable() {
        let app = Router::new().route(
            "/realms/test/account/credentials",
            get(|| async { axum::http::StatusCode::UNAUTHORIZED }),
        );
        let api = account_api(app).await;

        assert!(api.second_factor("access-1").await.is_err());
    }
}
