use serde::{Deserialize, Serialize};

use crate::identity::domain::{RoleName, UserId};

/// Claims carried by an access token. Part of the module's public API:
/// the auth guard hands these to any protected handler (also in other modules).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
	pub sub: i64,
	pub username: String,
	pub email: String,
	pub roles: Vec<RoleName>,
	pub exp: i64,
	pub iat: i64,
}

impl AccessClaims {
	pub fn user_id(&self) -> UserId {
		UserId(self.sub)
	}

	pub fn has_role(&self, role: RoleName) -> bool {
		self.roles.contains(&role)
	}
}
