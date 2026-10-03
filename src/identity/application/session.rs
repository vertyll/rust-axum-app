use jiff::{SignedDuration, Timestamp};
use serde::{Deserialize, Serialize};

use crate::identity::domain::{Email, KeycloakId, KeycloakIdentity, RoleName, User, UserId};

#[derive(Debug, Clone)]
pub struct TokenPair {
	pub access_token: String,
	pub refresh_token: String,
}

#[derive(Debug, Clone)]
pub struct VerifiedToken {
	pub identity: KeycloakIdentity,
	pub expires_at: Timestamp,
}

/// A signed-in browser: kept server-side in the session store, never sent to the browser.
#[derive(Clone, Serialize, Deserialize)]
pub struct AuthSession {
	pub identity: KeycloakIdentity,
	pub access_token: String,
	pub refresh_token: String,
	pub access_token_expires_at: Timestamp,
}

impl AuthSession {
	pub fn needs_refresh_at(&self, now: Timestamp, skew: SignedDuration) -> bool {
		now.checked_add(skew).unwrap_or(Timestamp::MAX) >= self.access_token_expires_at
	}
}

impl std::fmt::Debug for AuthSession {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("AuthSession")
			.field("identity", &self.identity)
			.field("access_token", &"***")
			.field("refresh_token", &"***")
			.field("access_token_expires_at", &self.access_token_expires_at)
			.finish()
	}
}

/// The authenticated caller handed to protected handlers, also in other modules.
#[derive(Debug, Clone)]
pub struct Caller {
	pub user_id: UserId,
	pub keycloak_id: KeycloakId,
	pub email: Email,
	pub roles: Vec<RoleName>,
}

impl Caller {
	pub fn has_role(&self, role: RoleName) -> bool {
		self.roles.contains(&role)
	}
}

impl From<&User> for Caller {
	fn from(user: &User) -> Self {
		Self {
			user_id: user.id,
			keycloak_id: user.keycloak_id.clone(),
			email: user.email.clone(),
			roles: user.roles.clone(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn session(expires_at: Timestamp) -> AuthSession {
		AuthSession {
			identity: KeycloakIdentity {
				keycloak_id: KeycloakId::new("subject"),
				email: Email::parse("ada@rust-axum-app.local").unwrap(),
				first_name: String::new(),
				last_name: String::new(),
				roles: vec![RoleName::User],
			},
			access_token: "access".into(),
			refresh_token: "refresh".into(),
			access_token_expires_at: expires_at,
		}
	}

	#[test]
	fn refreshes_only_within_the_skew_of_expiry() {
		let expiry: Timestamp = "2026-10-03T12:00:00Z".parse().unwrap();
		let skew = SignedDuration::from_secs(30);
		assert!(!session(expiry).needs_refresh_at(expiry - SignedDuration::from_secs(60), skew));
		assert!(session(expiry).needs_refresh_at(expiry - SignedDuration::from_secs(10), skew));
	}

	#[test]
	fn debug_output_never_shows_the_tokens() {
		let printed = format!("{:?}", session(Timestamp::UNIX_EPOCH));
		assert!(!printed.contains("\"access\""));
		assert!(printed.contains("***"));
	}
}
