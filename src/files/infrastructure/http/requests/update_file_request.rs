use serde::Deserialize;
use validator::Validate;

use crate::files::application::commands::UpdateFileMeta;
use crate::files::domain::StorageKind;
use crate::files::infrastructure::http::error::ApiError;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateFileRequest {
	pub original_name: Option<String>,
	pub mime_type: Option<String>,
	pub encoding: Option<String>,
	pub size: Option<i64>,
	#[validate(length(min = 1, message = "files.validators.file.storage_type.invalid"))]
	pub storage_type: Option<String>,
	pub url: Option<String>,
}

impl UpdateFileRequest {
	pub fn into_command(self) -> Result<UpdateFileMeta, ApiError> {
		Ok(UpdateFileMeta {
			original_name: self.original_name,
			mime_type: self.mime_type,
			encoding: self.encoding,
			size: self.size,
			storage: self
				.storage_type
				.map(|value| value.parse::<StorageKind>())
				.transpose()?,
			url: self.url,
		})
	}
}
