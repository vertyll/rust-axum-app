use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ForgotPasswordDto {
	#[validate(email)]
	pub email: String,
}
