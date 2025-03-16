use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ResetPasswordDto {
	pub token: String,

	#[validate(length(min = 8))]
	pub password: String,
}
