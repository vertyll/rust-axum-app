use crate::files::application::commands::UploadFile;
use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileRepository, FilesError, NewStoredFile, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub async fn upload(&self, cmd: UploadFile) -> Result<StoredFile, FilesError> {
		if cmd.data.is_empty() {
			return Err(FilesError::NoFileUploaded);
		}
		if cmd.storage != self.storage.kind() {
			return Err(FilesError::InvalidStorageType);
		}

		let size = cmd.data.len() as i64;
		let object = self.storage.store(cmd.data, &cmd.original_name).await?;
		let stored_path = object.file_path.clone();

		let new_file = NewStoredFile {
			filename: object.filename,
			original_name: cmd.original_name,
			file_path: object.file_path,
			mime_type: cmd.mime_type,
			encoding: "binary".to_string(),
			size,
			storage: cmd.storage,
			url: object.url,
			metadata: object.metadata,
		};

		match self.repository.create(new_file).await {
			Ok(file) => Ok(file),
			Err(err) => {
				// Compensation: the metadata row failed, remove the bytes.
				if let Err(cleanup) = self.storage.remove(&stored_path).await {
					tracing::error!("failed to clean up orphaned upload {stored_path}: {cleanup}");
				}
				Err(err)
			}
		}
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::testing::{files_harness, upload_cmd};
	use crate::files::domain::FilesError;

	#[tokio::test]
	async fn stores_bytes_then_metadata() {
		let (service, repository, storage) = files_harness();
		let file = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap();

		assert_eq!(file.size, 5);
		assert!(file.url.starts_with("/uploads/"));
		assert_eq!(storage.stored_paths().len(), 1);
		assert!(repository.get_any(file.id).is_some());
	}

	#[tokio::test]
	async fn rejects_empty_uploads() {
		let (service, _, storage) = files_harness();
		let err = service.upload(upload_cmd("empty.txt", b"")).await.unwrap_err();
		assert!(matches!(err, FilesError::NoFileUploaded));
		assert!(storage.stored_paths().is_empty());
	}

	#[tokio::test]
	async fn removes_bytes_when_persistence_fails() {
		let (service, repository, storage) = files_harness();
		repository.set_fail_create(true);

		let err = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap_err();
		assert!(matches!(err, FilesError::PersistenceFailure(_)));
		assert_eq!(storage.removed_paths(), storage.stored_paths());
	}
}
