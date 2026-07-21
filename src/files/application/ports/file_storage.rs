use super::StoredObject;
use crate::files::domain::{FilesError, StorageKind};

/// Byte storage for the files context.
pub trait FileStorage: Clone + Send + Sync + 'static {
	fn kind(&self) -> StorageKind;

	fn store(
		&self,
		data: Vec<u8>,
		original_name: &str,
	) -> impl Future<Output = Result<StoredObject, FilesError>> + Send;

	fn remove(&self, path: &str) -> impl Future<Output = Result<(), FilesError>> + Send;
}
