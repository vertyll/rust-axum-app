//! Keycloak adapters: the token endpoint, the JWKS-backed token verifier and PKCE.

mod client;
pub mod pkce;
mod shared_refresh;
mod verifier;

pub use client::HttpKeycloakClient;
pub use shared_refresh::SharedRefreshes;
pub use verifier::JwksTokenVerifier;

/// Where the realm is and how this application is registered in it.
#[derive(Debug, Clone)]
pub struct KeycloakSettings {
	pub realm_url: String,
	pub client_id: String,
	pub client_secret: String,
	pub audience: String,
	pub callback_url: String,
}

impl KeycloakSettings {
	pub fn endpoint(&self, name: &str) -> String {
		format!("{}/protocol/openid-connect/{name}", self.realm_url)
	}
}
