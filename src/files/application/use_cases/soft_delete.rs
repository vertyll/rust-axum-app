use jiff::Timestamp;

use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub async fn soft_delete(&self, id: FileId, by_user_id: i64) -> Result<(), FilesError> {
		let mut file = self.get(id).await?;
		file.soft_delete(by_user_id, Timestamp::now());
		self.repository.update(&file).await
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::testing::{files_harness, upload_cmd};

	#[tokio::test]
	async fn marks_who_deleted_and_keeps_the_row() {
		let (service, repository, storage) = files_harness();
		let file = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap();

		service.soft_delete(file.id, 42).await.unwrap();

		let stored = repository.get_any(file.id).unwrap();
		assert!(stored.is_deleted());
		assert_eq!(stored.deleted_by_user_id, Some(42));
		assert!(storage.removed_paths().is_empty());
	}
}
