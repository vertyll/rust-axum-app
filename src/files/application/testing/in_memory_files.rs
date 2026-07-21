use std::sync::{Arc, Mutex};

use jiff::Timestamp;

use crate::files::domain::{FileId, FileRepository, FilesError, NewStoredFile, StoredFile};

#[derive(Default)]
struct State {
	next_id: i64,
	files: Vec<StoredFile>,
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryFiles {
	state: Arc<Mutex<State>>,
	fail_create: Arc<Mutex<bool>>,
}

impl InMemoryFiles {
	pub fn set_fail_create(&self, fail: bool) {
		*self.fail_create.lock().unwrap() = fail;
	}

	/// Reads a row even when it is soft-deleted.
	pub fn get_any(&self, id: FileId) -> Option<StoredFile> {
		self.state.lock().unwrap().files.iter().find(|file| file.id == id).cloned()
	}
}

impl FileRepository for InMemoryFiles {
	async fn create(&self, file: NewStoredFile) -> Result<StoredFile, FilesError> {
		if *self.fail_create.lock().unwrap() {
			return Err(FilesError::PersistenceFailure("test create failure".into()));
		}
		let mut state = self.state.lock().unwrap();
		state.next_id += 1;
		let now = Timestamp::now();
		let stored = StoredFile {
			id: FileId(state.next_id),
			filename: file.filename,
			original_name: file.original_name,
			path: file.path,
			mime_type: file.mime_type,
			encoding: file.encoding,
			size: file.size,
			storage: file.storage,
			url: file.url,
			metadata: file.metadata,
			deleted_at: None,
			deleted_by_user_id: None,
			created_at: now,
			updated_at: now,
		};
		state.files.push(stored.clone());
		Ok(stored)
	}

	async fn find_active_by_id(&self, id: FileId) -> Result<Option<StoredFile>, FilesError> {
		let state = self.state.lock().unwrap();
		Ok(state.files.iter().find(|file| file.id == id && !file.is_deleted()).cloned())
	}

	async fn list_active(&self) -> Result<Vec<StoredFile>, FilesError> {
		let state = self.state.lock().unwrap();
		Ok(state.files.iter().filter(|file| !file.is_deleted()).cloned().collect())
	}

	async fn update(&self, file: &StoredFile) -> Result<(), FilesError> {
		let mut state = self.state.lock().unwrap();
		let index = state
			.files
			.iter()
			.position(|existing| existing.id == file.id)
			.ok_or_else(|| FilesError::PersistenceFailure("update of unknown file".into()))?;
		state.files[index] = file.clone();
		Ok(())
	}

	async fn delete(&self, id: FileId) -> Result<(), FilesError> {
		self.state.lock().unwrap().files.retain(|file| file.id != id);
		Ok(())
	}
}
