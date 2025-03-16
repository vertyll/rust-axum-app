use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct UpdateFileDto {
	pub original_name: Option<String>,
	pub mime_type: Option<String>,
	pub encoding: Option<String>,
	pub size: Option<i32>,
	pub storage_type: Option<String>,
	pub url: Option<String>,
}
