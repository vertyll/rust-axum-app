//! Keycloak adapters: the token endpoint, the admin API, the JWKS-backed
//! token verifier and PKCE.

mod admin;
mod client;
pub mod pkce;
mod shared_refresh;
mod verifier;

pub use admin::HttpRealmAdmin;
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
    #[must_use]
    pub fn endpoint(&self, name: &str) -> String {
        format!("{}/protocol/openid-connect/{name}", self.realm_url)
    }

    /// A path of the realm in the admin API: `…/realms/x` becomes `…/admin/realms/x`.
    #[must_use]
    pub fn admin_endpoint(&self, path: &str) -> String {
        let realm_url = self.realm_url.trim_end_matches('/');
        let admin = match realm_url.rfind("/realms/") {
            Some(at) => format!("{}/admin{}", &realm_url[..at], &realm_url[at..]),
            None => realm_url.to_owned(),
        };
        format!("{admin}/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::KeycloakSettings;

    #[test]
    fn the_admin_api_sits_beside_the_realm() {
        let settings = KeycloakSettings {
            realm_url: "http://localhost:8080/realms/rust-axum-app".into(),
            client_id: String::new(),
            client_secret: String::new(),
            audience: String::new(),
            callback_url: String::new(),
        };
        assert_eq!(
            settings.admin_endpoint("roles/ADMIN"),
            "http://localhost:8080/admin/realms/rust-axum-app/roles/ADMIN"
        );
    }
}
