use serde::Deserialize;
use validator::Validate;

use crate::identity::application::commands::RequestEmailChange;
use crate::identity::domain::Email;
use crate::identity::infrastructure::http::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct ChangeEmailRequest {
	#[validate(email)]
	pub email: String,
}

impl ChangeEmailRequest {
	pub fn into_command(self) -> Result<RequestEmailChange, ApiError> {
		Ok(RequestEmailChange { new_email: Email::parse(self.email)? })
	}
}
