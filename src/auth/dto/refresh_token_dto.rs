use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct RefreshTokenDto {
	pub refresh_token: String,
}
