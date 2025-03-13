use validator::ValidationError;

pub fn validate_username(username: &str) -> Result<(), ValidationError> {
	if username.chars().count() < 3 {
		let mut err = ValidationError::new("too_short");
		err.message = Some("Name must be at least 3 characters long.".into());
		return Err(err);
	}
	Ok(())
}
