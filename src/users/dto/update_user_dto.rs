use crate::users::validators::users_validators::validate_username;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateUserDto {
	#[validate(custom(function = "validate_username"))]
	pub username: Option<String>,

	#[validate(email)]
	pub email: Option<String>,
}
