use serde::Deserialize;
use validator::Validate;

use super::validate_username;
use crate::identity::application::commands::RegisterUser;
use crate::identity::domain::{Email, Username};
use crate::identity::infrastructure::http::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
	#[validate(custom(function = "validate_username"))]
	pub username: String,
	#[validate(email(message = "users.validators.email.invalid_format"))]
	pub email: String,
	#[validate(length(min = 8, message = "users.validators.password.too_short"))]
	pub password: String,
}

impl RegisterRequest {
	pub fn into_command(self) -> Result<RegisterUser, ApiError> {
		Ok(RegisterUser {
			username: Username::parse(self.username)?,
			email: Email::parse(self.email)?,
			password: self.password,
		})
	}
}
#[cfg(test)]
mod tests {
	use validator::Validate;

	use super::RegisterRequest;

	fn request(username: &str, email: &str, password: &str) -> RegisterRequest {
		RegisterRequest {
			username: username.into(),
			email: email.into(),
			password: password.into(),
		}
	}

	#[test]
	fn maps_into_domain_command() {
		let cmd = request("alice", "alice@example.com", "password123")
			.into_command()
			.unwrap();
		assert_eq!(cmd.username.as_str(), "alice");
		assert_eq!(cmd.email.as_str(), "alice@example.com");
	}

	#[test]
	fn validator_rejects_bad_transport_input() {
		assert!(request("al", "alice@example.com", "password123").validate().is_err());
		assert!(request("alice", "not-an-email", "password123").validate().is_err());
		assert!(request("alice", "alice@example.com", "short").validate().is_err());
	}
}
