use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateRoleDto {
	#[validate(length(min = 1, max = 50))]
	pub name: Option<String>,
	pub description: Option<String>,
}
