use std::sync::Arc;
use std::time::Duration;

use jiff::Timestamp;
use jsonwebtoken::jwk::JwkSet;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use serde::Deserialize;
use tokio::sync::RwLock;

use super::KeycloakSettings;
use crate::identity::application::ports::TokenVerifier;
use crate::identity::application::session::VerifiedToken;
use crate::identity::domain::{Email, IdentityError, KeycloakId, KeycloakIdentity, RoleName};

const TIMEOUT: Duration = Duration::from_secs(10);

/// Verifies access tokens against the realm's published keys. The key set
/// is fetched once and again only when a token names a key it does not
/// hold, which is how Keycloak's key rotation reaches the application.
#[derive(Clone)]
pub struct JwksTokenVerifier {
	http: reqwest::Client,
	jwks_url: String,
	validation: Arc<Validation>,
	keys: Arc<RwLock<Option<JwkSet>>>,
}

impl JwksTokenVerifier {
	pub fn new(settings: &KeycloakSettings) -> Result<Self, IdentityError> {
		let mut validation = Validation::new(Algorithm::RS256);
		validation.set_issuer(&[settings.realm_url()]);
		validation.set_audience(&[settings.audience.as_str()]);
		validation.set_required_spec_claims(&["exp", "sub", "iss", "aud"]);
		Self::with_validation(settings.endpoint("certs"), validation)
	}

	pub(crate) fn with_validation(jwks_url: String, validation: Validation) -> Result<Self, IdentityError> {
		let http = reqwest::Client::builder()
			.timeout(TIMEOUT)
			.build()
			.map_err(|err| IdentityError::IdentityProviderUnavailable(err.to_string()))?;
		Ok(Self {
			http,
			jwks_url,
			validation: Arc::new(validation),
			keys: Arc::default(),
		})
	}

	async fn key(&self, kid: &str) -> Result<DecodingKey, IdentityError> {
		if let Some(key) = self.cached(kid).await {
			return Ok(key);
		}
		let fetched: JwkSet = self
			.http
			.get(&self.jwks_url)
			.send()
			.await
			.and_then(reqwest::Response::error_for_status)
			.map_err(|err| IdentityError::IdentityProviderUnavailable(err.to_string()))?
			.json()
			.await
			.map_err(|err| IdentityError::IdentityProviderUnavailable(err.to_string()))?;
		*self.keys.write().await = Some(fetched);
		self.cached(kid).await.ok_or(IdentityError::InvalidToken)
	}

	async fn cached(&self, kid: &str) -> Option<DecodingKey> {
		let keys = self.keys.read().await;
		let jwk = keys.as_ref()?.find(kid)?;
		DecodingKey::from_jwk(jwk).ok()
	}
}

impl TokenVerifier for JwksTokenVerifier {
	async fn verify(&self, token: &str) -> Result<VerifiedToken, IdentityError> {
		let header = decode_header(token).map_err(|_| IdentityError::InvalidToken)?;
		let kid = header.kid.ok_or(IdentityError::InvalidToken)?;
		let key = self.key(&kid).await?;
		let claims = decode::<Claims>(token, &key, &self.validation)
			.map_err(|_| IdentityError::InvalidToken)?
			.claims;
		claims.into_verified()
	}
}

#[derive(Deserialize)]
struct Claims {
	sub: String,
	exp: i64,
	email: Option<String>,
	given_name: Option<String>,
	family_name: Option<String>,
	realm_access: Option<RealmAccess>,
}

#[derive(Deserialize)]
struct RealmAccess {
	#[serde(default)]
	roles: Vec<String>,
}

impl Claims {
	fn into_verified(self) -> Result<VerifiedToken, IdentityError> {
		let email =
			Email::parse(self.email.ok_or(IdentityError::InvalidToken)?).map_err(|_| IdentityError::InvalidToken)?;
		let mut roles: Vec<RoleName> = self
			.realm_access
			.map(|access| access.roles.iter().filter_map(|role| role.parse().ok()).collect())
			.unwrap_or_default();
		roles.sort_by_key(|role| role.as_str());
		roles.dedup();
		Ok(VerifiedToken {
			identity: KeycloakIdentity {
				keycloak_id: KeycloakId::new(self.sub),
				email,
				first_name: self.given_name.unwrap_or_default(),
				last_name: self.family_name.unwrap_or_default(),
				roles,
			},
			expires_at: Timestamp::from_second(self.exp).map_err(|_| IdentityError::InvalidToken)?,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn keeps_only_the_roles_the_application_knows() {
		let claims = Claims {
			sub: "subject".into(),
			exp: 1_800_000_000,
			email: Some("Ada@Rust-Axum-App.local".into()),
			given_name: Some("Ada".into()),
			family_name: None,
			realm_access: Some(RealmAccess {
				roles: vec![
					"ADMIN".into(),
					"offline_access".into(),
					"default-roles-rust-axum-app".into(),
					"USER".into(),
				],
			}),
		};

		let verified = claims.into_verified().unwrap();

		assert_eq!(verified.identity.roles, vec![RoleName::Admin, RoleName::User]);
		assert_eq!(verified.identity.email.as_str(), "ada@rust-axum-app.local");
		assert_eq!(verified.identity.last_name, "");
	}

	#[test]
	fn a_token_without_email_is_rejected() {
		let claims = Claims {
			sub: "subject".into(),
			exp: 1_800_000_000,
			email: None,
			given_name: None,
			family_name: None,
			realm_access: None,
		};
		assert!(matches!(claims.into_verified(), Err(IdentityError::InvalidToken)));
	}
}
