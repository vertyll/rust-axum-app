use jiff::Timestamp;
use toasty::stmt::Json;

use crate::files::domain::{FileId, FilesError, StoredFile};

#[derive(Debug, toasty::Model)]
#[table = "files"]
pub struct FileRecord {
	#[key]
	#[auto]
	pub id: i64,
	pub filename: String,
	pub original_name: String,
	pub file_path: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i64,
	pub storage_type: String,
	pub url: String,
	#[column(type = text)]
	pub metadata: Json<serde_json::Value>,
	pub deleted_at: Option<Timestamp>,
	#[index]
	pub deleted_by_user_id: Option<i64>,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,
}

impl FileRecord {
	pub fn to_domain(&self) -> Result<StoredFile, FilesError> {
		Ok(StoredFile {
			id: FileId(self.id),
			filename: self.filename.clone(),
			original_name: self.original_name.clone(),
			file_path: self.file_path.clone(),
			mime_type: self.mime_type.clone(),
			encoding: self.encoding.clone(),
			size: self.size,
			storage: self.storage_type.parse().map_err(|_| {
				FilesError::CorruptData(format!("file {}: unknown storage type {}", self.id, self.storage_type))
			})?,
			url: self.url.clone(),
			metadata: self.metadata.0.clone(),
			deleted_at: self.deleted_at,
			deleted_by_user_id: self.deleted_by_user_id,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}
