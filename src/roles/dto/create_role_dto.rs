use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRoleDto {
	#[validate(length(min = 1, max = 50))]
	pub name: String,
	pub description: Option<String>,
}
