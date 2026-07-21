use std::sync::{Arc, Mutex};

use crate::files::application::ports::{FileStorage, StoredObject};
use crate::files::domain::{FilesError, StorageKind};

#[derive(Clone, Default)]
pub(crate) struct FakeStorage {
	stored: Arc<Mutex<Vec<String>>>,
	removed: Arc<Mutex<Vec<String>>>,
}

impl FakeStorage {
	pub fn stored_paths(&self) -> Vec<String> {
		self.stored.lock().unwrap().clone()
	}

	pub fn removed_paths(&self) -> Vec<String> {
		self.removed.lock().unwrap().clone()
	}
}

impl FileStorage for FakeStorage {
	fn kind(&self) -> StorageKind {
		StorageKind::Local
	}

	async fn store(&self, data: Vec<u8>, original_name: &str) -> Result<StoredObject, FilesError> {
		let path = format!("/fake/{original_name}");
		self.stored.lock().unwrap().push(path.clone());
		Ok(StoredObject {
			filename: format!("generated-{original_name}"),
			path,
			url: format!("/uploads/generated-{original_name}"),
			metadata: serde_json::json!({ "bytes": data.len() }),
		})
	}

	async fn remove(&self, path: &str) -> Result<(), FilesError> {
		self.removed.lock().unwrap().push(path.to_string());
		Ok(())
	}
}
