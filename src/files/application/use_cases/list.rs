use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileRepository, FilesError, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub async fn list(&self) -> Result<Vec<StoredFile>, FilesError> {
		self.repository.list_active().await
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::testing::{files_harness, upload_cmd};

	#[tokio::test]
	async fn excludes_soft_deleted_files() {
		let (service, _, _) = files_harness();
		let kept = service.upload(upload_cmd("keep.txt", b"a")).await.unwrap();
		let gone = service.upload(upload_cmd("gone.txt", b"b")).await.unwrap();
		service.soft_delete(gone.id, 7).await.unwrap();

		let listed = service.list().await.unwrap();
		assert_eq!(listed.len(), 1);
		assert_eq!(listed[0].id, kept.id);
	}
}
