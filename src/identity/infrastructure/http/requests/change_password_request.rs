use serde::Deserialize;
use validator::Validate;

use crate::identity::application::commands::ChangePassword;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangePasswordRequest {
	#[validate(length(min = 8))]
	pub current_password: String,
	#[validate(length(min = 8))]
	pub new_password: String,
}

impl From<ChangePasswordRequest> for ChangePassword {
	fn from(request: ChangePasswordRequest) -> Self {
		ChangePassword {
			current_password: request.current_password,
			new_password: request.new_password,
		}
	}
}
