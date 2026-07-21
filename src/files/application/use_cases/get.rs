use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub async fn get(&self, id: FileId) -> Result<StoredFile, FilesError> {
		self.repository.find_active_by_id(id).await?.ok_or(FilesError::NotFound)
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::testing::{files_harness, upload_cmd};
	use crate::files::domain::{FileId, FilesError};

	#[tokio::test]
	async fn missing_and_soft_deleted_files_are_invisible() {
		let (service, repository, _) = files_harness();
		let missing = service.get(FileId(999)).await.unwrap_err();
		assert!(matches!(missing, FilesError::NotFound));

		let file = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap();
		service.soft_delete(file.id, 7).await.unwrap();

		let hidden = service.get(file.id).await.unwrap_err();
		assert!(matches!(hidden, FilesError::NotFound));
		assert!(repository.get_any(file.id).is_some());
	}
}
