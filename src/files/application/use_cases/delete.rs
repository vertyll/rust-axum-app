use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	/// Removes both the bytes and the metadata row.
	pub async fn delete(&self, id: FileId) -> Result<(), FilesError> {
		let file = self.get(id).await?;
		self.storage.remove(&file.file_path).await?;
		self.repository.delete(id).await
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::testing::{files_harness, upload_cmd};

	#[tokio::test]
	async fn removes_bytes_and_metadata() {
		let (service, repository, storage) = files_harness();
		let file = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap();

		service.delete(file.id).await.unwrap();
		assert_eq!(storage.removed_paths(), vec![file.file_path]);
		assert!(repository.get_any(file.id).is_none());
	}
}
