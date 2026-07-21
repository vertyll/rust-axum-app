//! Inbound request DTOs, one file per struct: transport validation
//! (`validator`), then `into_command` parses the domain value objects on
//! the way in.

mod change_email_request;
mod change_password_request;
mod forgot_password_request;
mod login_request;
mod register_request;
mod reset_password_request;
mod token_query;
mod update_user_request;

pub use change_email_request::ChangeEmailRequest;
pub use change_password_request::ChangePasswordRequest;
pub use forgot_password_request::ForgotPasswordRequest;
pub use login_request::LoginRequest;
pub use register_request::RegisterRequest;
pub use reset_password_request::ResetPasswordRequest;
pub use token_query::TokenQuery;
pub use update_user_request::UpdateUserRequest;

use std::borrow::Cow;

use validator::ValidationError;

use crate::identity::domain::Username;
use crate::shared_infrastructure::i18n::translate;

fn validate_username(username: &str) -> Result<(), ValidationError> {
	if username.chars().count() < Username::MIN_LENGTH {
		let mut error = ValidationError::new("too_short");
		error.message = Some(Cow::Owned(translate("users.validators.username.too_short")));
		return Err(error);
	}
	Ok(())
}
