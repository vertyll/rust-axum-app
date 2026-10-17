//! Repository port of the files context (Toasty adapter in
//! `infrastructure::persistence`). Soft-deleted files are invisible to all
//! reads by contract.

use jiff::Timestamp;

use super::error::FilesError;
use super::file::{FileId, NewStoredFile, StoredFile};
use crate::shared_kernel::pagination::{PageRequest, PageResult};

pub trait FileRepository: Clone + Send + Sync + 'static {
    fn create(&self, file: NewStoredFile) -> impl Future<Output = Result<StoredFile, FilesError>> + Send;

    /// The file, pending or uploaded.
    fn find_active_by_id(&self, id: FileId) -> impl Future<Output = Result<Option<StoredFile>, FilesError>> + Send;

    /// The uploaded files, newest first.
    fn page_uploaded(
        &self,
        request: PageRequest,
    ) -> impl Future<Output = Result<PageResult<StoredFile>, FilesError>> + Send;

    /// The uploads still pending that were allowed before `before`.
    fn pending_before(&self, before: Timestamp) -> impl Future<Output = Result<Vec<StoredFile>, FilesError>> + Send;

    fn update(&self, file: &StoredFile) -> impl Future<Output = Result<(), FilesError>> + Send;

    /// Hard delete of the metadata row (bytes are removed by the storage adapter beforehand).
    fn delete(&self, id: FileId) -> impl Future<Output = Result<(), FilesError>> + Send;
}
