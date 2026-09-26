use serde::Deserialize;
use validator::Validate;

use super::validate_username;
use crate::identity::application::commands::UpdateUserProfile;
use crate::identity::domain::{Email, Username};
use crate::identity::infrastructure::http::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateUserRequest {
	#[validate(custom(function = "validate_username"))]
	pub username: Option<String>,
	#[validate(email(message = "users.validators.email.invalid_format"))]
	pub email: Option<String>,
}

impl UpdateUserRequest {
	pub fn into_command(self) -> Result<UpdateUserProfile, ApiError> {
		Ok(UpdateUserProfile {
			username: self.username.map(Username::parse).transpose()?,
			email: self.email.map(Email::parse).transpose()?,
		})
	}
}
