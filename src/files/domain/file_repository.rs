//! Repository port of the files context (Toasty adapter in
//! `infrastructure::persistence`). Soft-deleted files are invisible to all
//! reads by contract.

use super::error::FilesError;
use super::file::{FileId, NewStoredFile, StoredFile};

pub trait FileRepository: Clone + Send + Sync + 'static {
	fn create(
		&self,
		file: NewStoredFile,
	) -> impl Future<Output = Result<StoredFile, FilesError>> + Send;

	fn find_active_by_id(
		&self,
		id: FileId,
	) -> impl Future<Output = Result<Option<StoredFile>, FilesError>> + Send;

	fn list_active(&self) -> impl Future<Output = Result<Vec<StoredFile>, FilesError>> + Send;

	fn update(&self, file: &StoredFile) -> impl Future<Output = Result<(), FilesError>> + Send;

	/// Hard delete of the metadata row (bytes are removed by the storage adapter beforehand).
	fn delete(&self, id: FileId) -> impl Future<Output = Result<(), FilesError>> + Send;
}
