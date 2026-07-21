use serde::Deserialize;
use validator::Validate;

use crate::identity::application::commands::RequestPasswordReset;
use crate::identity::domain::Email;
use crate::identity::infrastructure::http::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
	#[validate(email)]
	pub email: String,
}

impl ForgotPasswordRequest {
	pub fn into_command(self) -> Result<RequestPasswordReset, ApiError> {
		Ok(RequestPasswordReset {
			email: Email::parse(self.email)?,
		})
	}
}
