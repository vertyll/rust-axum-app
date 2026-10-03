use jiff::Timestamp;
use serde::Serialize;

use crate::identity::application::AuthSession;
use crate::identity::domain::{RoleName, User};

#[derive(Debug, Serialize)]
pub struct UserResponse {
	pub id: i64,
	pub keycloak_id: String,
	pub email: String,
	pub first_name: String,
	pub last_name: String,
	pub roles: Vec<RoleName>,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
}

impl From<User> for UserResponse {
	fn from(user: User) -> Self {
		Self {
			id: user.id.0,
			keycloak_id: user.keycloak_id.to_string(),
			email: user.email.to_string(),
			first_name: user.first_name,
			last_name: user.last_name,
			roles: user.roles,
			created_at: user.created_at,
			updated_at: user.updated_at,
		}
	}
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
	pub user_id: String,
	pub email: String,
	pub roles: Vec<RoleName>,
}

impl From<&AuthSession> for SessionResponse {
	fn from(session: &AuthSession) -> Self {
		Self {
			user_id: session.identity.keycloak_id.to_string(),
			email: session.identity.email.to_string(),
			roles: session.identity.roles.clone(),
		}
	}
}
