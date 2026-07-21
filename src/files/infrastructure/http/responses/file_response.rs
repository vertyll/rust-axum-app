use jiff::Timestamp;
use serde::Serialize;

use crate::files::domain::{StorageKind, StoredFile};

#[derive(Debug, Serialize)]
pub struct FileResponse {
	pub id: i64,
	pub filename: String,
	pub original_name: String,
	pub path: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i64,
	pub storage_type: StorageKind,
	pub url: String,
	pub metadata: serde_json::Value,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
}

impl From<StoredFile> for FileResponse {
	fn from(file: StoredFile) -> Self {
		Self {
			id: file.id.0,
			filename: file.filename,
			original_name: file.original_name,
			path: file.path,
			mime_type: file.mime_type,
			encoding: file.encoding,
			size: file.size,
			storage_type: file.storage,
			url: file.url,
			metadata: file.metadata,
			created_at: file.created_at,
			updated_at: file.updated_at,
		}
	}
}
