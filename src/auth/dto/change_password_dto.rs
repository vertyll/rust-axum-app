use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ChangePasswordDto {
	#[validate(length(min = 8))]
	pub current_password: String,

	#[validate(length(min = 8))]
	pub new_password: String,
}
