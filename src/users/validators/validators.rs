use crate::i18n::setup::translate;
use std::borrow::Cow;
use validator::ValidationError;

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
	if username.chars().count() < 3 {
		let mut err = ValidationError::new("too_short");
		err.message = Some(Cow::Owned(translate("users.validators.username.too_short")));
		return Err(err);
	}
	Ok(())
}

pub fn validate_password(password: &str) -> Result<(), ValidationError> {
	if password.chars().count() < 8 {
		let mut err = ValidationError::new("too_short");
		err.message = Some(Cow::Owned(translate("users.validators.password.too_short")));
		return Err(err);
	}

	if !password.chars().any(|c| c.is_ascii_digit()) {
		let mut err = ValidationError::new("no_digit");
		err.message = Some(Cow::Owned(translate("users.validators.password.no_digit")));
		return Err(err);
	}

	Ok(())
}

pub fn validate_email(email: &str) -> Result<(), ValidationError> {
	// Simple check for '@' character
	if !email.contains('@') {
		let mut err = ValidationError::new("invalid_format");
		err.message = Some(Cow::Owned(translate("users.validators.email.invalid_format")));
		return Err(err);
	}
	Ok(())
}
