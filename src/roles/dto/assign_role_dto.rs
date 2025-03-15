use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct AssignRoleDto {
	pub user_id: i32,
	pub role_id: i32,
}
