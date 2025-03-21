use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct ChangeEmailDto {
	#[validate(email)]
	pub email: String,
}
