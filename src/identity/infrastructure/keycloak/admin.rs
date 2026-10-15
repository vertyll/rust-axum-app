use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{Method, RequestBuilder, Response, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use super::KeycloakSettings;
use crate::identity::application::ports::RealmAdmin;
use crate::identity::domain::{IdentityError, KeycloakId, RoleName};

const TIMEOUT: Duration = Duration::from_secs(10);
const EXPIRY_SKEW: Duration = Duration::from_secs(30);

/// Keycloak's admin REST API, called as this application's service account,
/// which the realm grants `manage-realm` and `manage-users` of
/// `realm-management`. Its token is kept until shortly before it expires.
#[derive(Clone)]
pub struct HttpRealmAdmin {
    http: reqwest::Client,
    settings: Arc<KeycloakSettings>,
    token: Arc<Mutex<Option<(String, Instant)>>>,
}

#[derive(Deserialize)]
struct ServiceToken {
    access_token: String,
    expires_in: u64,
}

/// A realm role as the admin API reads and writes it.
#[derive(Serialize, Deserialize)]
struct RealmRole {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<String>,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
}

impl HttpRealmAdmin {
    pub fn new(settings: KeycloakSettings) -> Result<Self, IdentityError> {
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .build()
            .map_err(unavailable)?;
        Ok(Self {
            http,
            settings: Arc::new(settings),
            token: Arc::default(),
        })
    }

    async fn token(&self) -> Result<String, IdentityError> {
        let mut cached = self.token.lock().await;
        if let Some((token, expires)) = cached.as_ref()
            && Instant::now() + EXPIRY_SKEW < *expires
        {
            return Ok(token.clone());
        }
        let response = self
            .http
            .post(self.settings.endpoint("token"))
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", self.settings.client_id.as_str()),
                ("client_secret", self.settings.client_secret.as_str()),
            ])
            .send()
            .await
            .map_err(unavailable)?;
        let body: ServiceToken = checked(response)?.json().await.map_err(unavailable)?;
        let expires = Instant::now() + Duration::from_secs(body.expires_in);
        *cached = Some((body.access_token.clone(), expires));
        Ok(body.access_token)
    }

    async fn request(&self, method: Method, path: &str) -> Result<RequestBuilder, IdentityError> {
        let token = self.token().await?;
        Ok(self
            .http
            .request(method, self.settings.admin_endpoint(path))
            .bearer_auth(token))
    }

    async fn realm_role(&self, name: &RoleName) -> Result<RealmRole, IdentityError> {
        let response = self
            .request(Method::GET, &format!("roles/{}", name.realm_role()))
            .await?
            .send()
            .await
            .map_err(unavailable)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(IdentityError::RoleNotFound);
        }
        checked(response)?.json().await.map_err(unavailable)
    }

    async fn role_mapping(&self, method: Method, account: &KeycloakId, name: &RoleName) -> Result<(), IdentityError> {
        let role = self.realm_role(name).await?;
        let response = self
            .request(method, &format!("users/{account}/role-mappings/realm"))
            .await?
            .json(&[role])
            .send()
            .await
            .map_err(unavailable)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Err(IdentityError::UserNotFound);
        }
        checked(response).map(drop)
    }
}

impl RealmAdmin for HttpRealmAdmin {
    async fn create_role(&self, name: &RoleName, description: &str) -> Result<(), IdentityError> {
        let role = RealmRole {
            id: None,
            name: name.realm_role(),
            description: Some(description.to_owned()),
        };
        let response = self
            .request(Method::POST, "roles")
            .await?
            .json(&role)
            .send()
            .await
            .map_err(unavailable)?;
        if response.status() == StatusCode::CONFLICT {
            return Ok(());
        }
        checked(response).map(drop)
    }

    async fn delete_role(&self, name: &RoleName) -> Result<(), IdentityError> {
        let response = self
            .request(Method::DELETE, &format!("roles/{}", name.realm_role()))
            .await?
            .send()
            .await
            .map_err(unavailable)?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        checked(response).map(drop)
    }

    async fn assign_role(&self, account: &KeycloakId, name: &RoleName) -> Result<(), IdentityError> {
        self.role_mapping(Method::POST, account, name).await
    }

    async fn remove_role(&self, account: &KeycloakId, name: &RoleName) -> Result<(), IdentityError> {
        self.role_mapping(Method::DELETE, account, name).await
    }
}

/// The response when Keycloak did what was asked; any refusal means the realm
/// or the service account is not set up as this application needs.
fn checked(response: Response) -> Result<Response, IdentityError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    let url = response.url().path().to_owned();
    Err(IdentityError::IdentityProviderUnavailable(format!(
        "admin API answered {status} for {url}"
    )))
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "passed to `map_err`, which hands over the error it maps"
)]
fn unavailable(err: reqwest::Error) -> IdentityError {
    IdentityError::IdentityProviderUnavailable(err.to_string())
}
