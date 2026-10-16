use std::fmt;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::error::IdentityError;
use super::role::RoleName;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The subject Keycloak gives an account; the join between the two systems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeycloakId(String);

impl KeycloakId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for KeycloakId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email(String);

impl Email {
    pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
        let value = value.into().trim().to_lowercase();
        let valid = value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.') && !domain.starts_with('.'));
        if valid {
            Ok(Self(value))
        } else {
            Err(IdentityError::InvalidEmail)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What a verified Keycloak access token says about its holder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeycloakIdentity {
    pub keycloak_id: KeycloakId,
    pub email: Email,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<RoleName>,
}

#[derive(Debug, Clone)]
pub struct NewUser {
    pub keycloak_id: KeycloakId,
    pub email: Email,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<RoleName>,
}

impl From<&KeycloakIdentity> for NewUser {
    fn from(identity: &KeycloakIdentity) -> Self {
        Self {
            keycloak_id: identity.keycloak_id.clone(),
            email: identity.email.clone(),
            first_name: identity.first_name.clone(),
            last_name: identity.last_name.clone(),
            roles: identity.roles.clone(),
        }
    }
}

/// An account mirrored from Keycloak. Keycloak owns the credentials; this
/// copy exists so the application can join its own data to a person.
#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub keycloak_id: KeycloakId,
    pub email: Email,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<RoleName>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl User {
    #[must_use]
    pub fn has_role(&self, role: &RoleName) -> bool {
        self.roles.contains(role)
    }

    /// Mirrors the identity; answers whether anything changed, so an
    /// unchanged account costs no write.
    pub fn mirror(&mut self, identity: &KeycloakIdentity) -> bool {
        let changed = self.email != identity.email
            || self.first_name != identity.first_name
            || self.last_name != identity.last_name
            || self.roles != identity.roles;
        if changed {
            self.email = identity.email.clone();
            self.first_name.clone_from(&identity.first_name);
            self.last_name.clone_from(&identity.last_name);
            self.roles.clone_from(&identity.roles);
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity(email: &str, roles: Vec<RoleName>) -> KeycloakIdentity {
        KeycloakIdentity {
            keycloak_id: KeycloakId::new("subject"),
            email: Email::parse(email).unwrap(),
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            roles,
        }
    }

    fn role(name: &str) -> RoleName {
        RoleName::parse(name).unwrap()
    }

    fn user() -> User {
        let identity = identity("ada@rust-axum-app.local", vec![role("user")]);
        User {
            id: UserId(1),
            keycloak_id: identity.keycloak_id,
            email: identity.email,
            first_name: identity.first_name,
            last_name: identity.last_name,
            roles: identity.roles,
            created_at: Timestamp::UNIX_EPOCH,
            updated_at: Timestamp::UNIX_EPOCH,
        }
    }

    #[test]
    fn email_is_normalised_and_checked() {
        assert_eq!(Email::parse(" Ada@Example.COM ").unwrap().as_str(), "ada@example.com");
        assert!(Email::parse("no-at-sign").is_err());
        assert!(Email::parse("ada@localhost").is_err());
    }

    #[test]
    fn mirroring_an_unchanged_identity_reports_no_change() {
        let mut user = user();
        assert!(!user.mirror(&identity("ada@rust-axum-app.local", vec![role("user")])));
    }

    #[test]
    fn mirroring_takes_the_new_email_and_roles() {
        let mut user = user();
        assert!(user.mirror(&identity("ada@new.example", vec![role("admin")])));
        assert_eq!(user.email.as_str(), "ada@new.example");
        assert!(user.has_role(&role("admin")));
        assert!(!user.has_role(&role("user")));
    }
}
