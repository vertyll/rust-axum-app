//! Local-filesystem adapter for `FileStorage`; all IO goes through `tokio::fs`.

use std::ffi::OsStr;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use jiff::Timestamp;
use uuid::Uuid;

use crate::files::application::{FileStorage, StoredObject};
use crate::files::domain::{FilesError, StorageKind};

#[derive(Clone)]
pub struct LocalFileStorage {
	upload_dir: PathBuf,
	base_url: String,
}

impl LocalFileStorage {
	pub fn new(upload_dir: impl Into<PathBuf>, base_url: impl Into<String>) -> Self {
		Self {
			upload_dir: upload_dir.into(),
			base_url: base_url.into().trim_end_matches('/').to_string(),
		}
	}
}

impl FileStorage for LocalFileStorage {
	fn kind(&self) -> StorageKind {
		StorageKind::Local
	}

	async fn store(&self, data: Vec<u8>, original_name: &str) -> Result<StoredObject, FilesError> {
		tokio::fs::create_dir_all(&self.upload_dir)
			.await
			.map_err(|err| storage("create upload directory", err))?;

		let extension = Path::new(original_name)
			.extension()
			.and_then(OsStr::to_str)
			.unwrap_or_default();
		let filename = if extension.is_empty() {
			Uuid::new_v4().to_string()
		} else {
			format!("{}.{extension}", Uuid::new_v4())
		};

		let full_path = self.upload_dir.join(&filename);
		tokio::fs::write(&full_path, &data)
			.await
			.map_err(|err| storage("write file", err))?;

		let metadata = serde_json::json!({
			"size": data.len(),
			"stored_at": Timestamp::now().to_string(),
		});

		Ok(StoredObject {
			url: format!("{}/{filename}", self.base_url),
			file_path: full_path.to_string_lossy().into_owned(),
			filename,
			metadata,
		})
	}

	async fn remove(&self, path: &str) -> Result<(), FilesError> {
		match tokio::fs::remove_file(path).await {
			Ok(()) => Ok(()),
			Err(err) if err.kind() == ErrorKind::NotFound => Ok(()),
			Err(err) => Err(storage("delete file", err)),
		}
	}
}

fn storage(action: &str, err: std::io::Error) -> FilesError {
	tracing::error!("storage: failed to {action}: {err}");
	FilesError::StorageFailure(action.to_string())
}
#[cfg(test)]
mod tests {
	use super::LocalFileStorage;
	use crate::files::application::ports::FileStorage;

	#[tokio::test]
	async fn store_then_remove_round_trip() {
		let dir = std::env::temp_dir().join(format!("rust-axum-app-test-{}", uuid::Uuid::new_v4()));
		tokio::fs::create_dir_all(&dir).await.unwrap();
		let storage = LocalFileStorage::new(dir.clone(), "/uploads");

		let object = storage.store(b"hello".to_vec(), "note.txt").await.unwrap();
		assert!(object.url.starts_with("/uploads/"));
		assert!(tokio::fs::metadata(&object.file_path).await.is_ok());

		storage.remove(&object.file_path).await.unwrap();
		assert!(tokio::fs::metadata(&object.file_path).await.is_err());

		tokio::fs::remove_dir_all(&dir).await.ok();
	}
}
