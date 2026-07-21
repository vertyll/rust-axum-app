use serde::Deserialize;
use validator::Validate;

use crate::identity::application::commands::ResetPassword;

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
	pub token: String,
	#[validate(length(min = 8))]
	pub password: String,
}

impl From<ResetPasswordRequest> for ResetPassword {
	fn from(request: ResetPasswordRequest) -> Self {
		ResetPassword { token: request.token, new_password: request.password }
	}
}
