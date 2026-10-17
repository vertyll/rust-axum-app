use std::sync::{Arc, Mutex};

use crate::shared_kernel::pagination::{PageRequest, PageResult};
use jiff::Timestamp;

use crate::files::domain::{FileId, FileRepository, FileStatus, FilesError, NewStoredFile, StoredFile};

#[derive(Default)]
struct State {
    next_id: i64,
    files: Vec<StoredFile>,
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryFiles {
    state: Arc<Mutex<State>>,
}

impl InMemoryFiles {
    /// Reads a row even when it is soft-deleted.
    pub fn get_any(&self, id: FileId) -> Option<StoredFile> {
        self.state
            .lock()
            .unwrap()
            .files
            .iter()
            .find(|file| file.id == id)
            .cloned()
    }
}

impl FileRepository for InMemoryFiles {
    async fn create(&self, file: NewStoredFile) -> Result<StoredFile, FilesError> {
        let mut state = self.state.lock().unwrap();
        state.next_id += 1;
        let now = Timestamp::now();
        let stored = StoredFile {
            id: FileId(state.next_id),
            object_key: file.object_key,
            original_name: file.original_name,
            mime_type: file.mime_type,
            encoding: "binary".to_string(),
            size: file.size,
            storage: file.storage,
            status: FileStatus::Pending,
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
        Ok(state
            .files
            .iter()
            .find(|file| file.id == id && !file.is_deleted())
            .cloned())
    }

    async fn page_uploaded(&self, request: PageRequest) -> Result<PageResult<StoredFile>, FilesError> {
        let state = self.state.lock().unwrap();
        let mut uploaded: Vec<StoredFile> = state
            .files
            .iter()
            .filter(|file| !file.is_deleted() && file.status == FileStatus::Uploaded)
            .cloned()
            .collect();
        uploaded.sort_by_key(|file| std::cmp::Reverse(file.id.0));
        Ok(request.slice(uploaded))
    }

    async fn pending_before(&self, before: Timestamp) -> Result<Vec<StoredFile>, FilesError> {
        let state = self.state.lock().unwrap();
        Ok(state
            .files
            .iter()
            .filter(|file| file.status == FileStatus::Pending && file.created_at < before)
            .cloned()
            .collect())
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
