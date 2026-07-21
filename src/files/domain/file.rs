//! Domain layer of the file's context.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::error::FilesError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub i64);

impl fmt::Display for FileId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

/// Where a file's bytes live. Currently only local disk; adding S3 later
/// means a new variant plus a new `FileStorage` adapter — nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
	Local,
}

impl StorageKind {
	pub const fn as_str(self) -> &'static str {
		match self {
			StorageKind::Local => "local",
		}
	}
}

impl FromStr for StorageKind {
	type Err = FilesError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value.to_ascii_lowercase().as_str() {
			"local" => Ok(StorageKind::Local),
			_ => Err(FilesError::InvalidStorageType),
		}
	}
}

impl fmt::Display for StorageKind {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}

/// A stored file's metadata as the business sees it.
#[derive(Debug, Clone)]
pub struct StoredFile {
	pub id: FileId,
	pub filename: String,
	pub original_name: String,
	pub file_path: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i64,
	pub storage: StorageKind,
	pub url: String,
	pub metadata: serde_json::Value,
	pub deleted_at: Option<Timestamp>,
	/// Cross-context reference by id only — no ORM relation into the
	/// identity module, so the module boundary stays real.
	pub deleted_by_user_id: Option<i64>,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
}

impl StoredFile {
	pub fn is_deleted(&self) -> bool {
		self.deleted_at.is_some()
	}

	pub fn soft_delete(&mut self, by_user_id: i64, now: Timestamp) {
		self.deleted_at = Some(now);
		self.deleted_by_user_id = Some(by_user_id);
	}
}

/// A file that has been written to storage but not yet persisted.
#[derive(Debug, Clone)]
pub struct NewStoredFile {
	pub filename: String,
	pub original_name: String,
	pub file_path: String,
	pub mime_type: String,
	pub encoding: String,
	pub size: i64,
	pub storage: StorageKind,
	pub url: String,
	pub metadata: serde_json::Value,
}
#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use super::*;

	#[test]
	fn storage_kind_parses_case_insensitively() {
		assert_eq!("Local".parse::<StorageKind>(), Ok(StorageKind::Local));
		assert!(matches!(
			"s3".parse::<StorageKind>(),
			Err(FilesError::InvalidStorageType)
		));
	}

	#[test]
	fn soft_delete_marks_who_and_when() {
		let mut file = StoredFile {
			id: FileId(1),
			filename: "gen.txt".into(),
			original_name: "note.txt".into(),
			file_path: "/tmp/gen.txt".into(),
			mime_type: "text/plain".into(),
			encoding: "binary".into(),
			size: 5,
			storage: StorageKind::Local,
			url: "/uploads/gen.txt".into(),
			metadata: serde_json::json!({}),
			deleted_at: None,
			deleted_by_user_id: None,
			created_at: Timestamp::UNIX_EPOCH,
			updated_at: Timestamp::UNIX_EPOCH,
		};
		assert!(!file.is_deleted());

		file.soft_delete(42, Timestamp::now());
		assert!(file.is_deleted());
		assert_eq!(file.deleted_by_user_id, Some(42));
	}
}
