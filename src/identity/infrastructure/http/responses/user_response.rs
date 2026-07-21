use jiff::Timestamp;
use serde::Serialize;

use crate::identity::domain::User;

#[derive(Debug, Serialize)]
pub struct UserResponse {
	pub id: i64,
	pub username: String,
	pub email: String,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
}

impl From<User> for UserResponse {
	fn from(user: User) -> Self {
		Self {
			id: user.id.0,
			username: user.username.to_string(),
			email: user.email.to_string(),
			created_at: user.created_at,
			updated_at: user.updated_at,
		}
	}
}
