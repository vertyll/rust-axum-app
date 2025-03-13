use crate::users::validators::validators::validate_username;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateUserDto {
	#[validate(custom(function = "validate_username"))]
	pub username: String,

	#[validate(email)]
	pub email: String,

	#[validate(length(min = 8))]
	pub password: String,
}
