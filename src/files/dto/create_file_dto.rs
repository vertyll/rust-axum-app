use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateFileDto {
	pub original_name: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i32,
	#[validate(length(min = 1))]
	pub storage_type: Option<String>,
}
