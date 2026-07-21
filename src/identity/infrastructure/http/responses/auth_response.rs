use serde::Serialize;

use super::UserResponse;

#[derive(Debug, Serialize)]
pub struct AuthResponse {
	pub user: UserResponse,
	pub access_token: String,
}
