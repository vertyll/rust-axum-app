use serde::Deserialize;
use validator::Validate;

use crate::identity::application::commands::Credentials;

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
	pub username: String,
	pub password: String,
}

impl From<LoginRequest> for Credentials {
	fn from(request: LoginRequest) -> Self {
		Credentials {
			username: request.username,
			password: request.password,
		}
	}
}
