use crate::files::application::commands::UpdateFileMeta;
use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
	pub async fn update(
		&self,
		id: FileId,
		changes: UpdateFileMeta,
	) -> Result<StoredFile, FilesError> {
		let mut file = self.get(id).await?;

		if let Some(original_name) = changes.original_name {
			file.original_name = original_name;
		}
		if let Some(mime_type) = changes.mime_type {
			file.mime_type = mime_type;
		}
		if let Some(encoding) = changes.encoding {
			file.encoding = encoding;
		}
		if let Some(size) = changes.size {
			file.size = size;
		}
		if let Some(storage) = changes.storage {
			file.storage = storage;
		}
		if let Some(url) = changes.url {
			file.url = url;
		}

		self.repository.update(&file).await?;
		Ok(file)
	}
}
#[cfg(test)]
mod tests {
	use crate::files::application::commands::UpdateFileMeta;
	use crate::files::application::testing::{files_harness, upload_cmd};

	#[tokio::test]
	async fn patches_only_the_provided_fields() {
		let (service, _, _) = files_harness();
		let file = service.upload(upload_cmd("note.txt", b"hello")).await.unwrap();

		let changes =
			UpdateFileMeta { original_name: Some("renamed.txt".into()), ..Default::default() };
		let updated = service.update(file.id, changes).await.unwrap();

		assert_eq!(updated.original_name, "renamed.txt");
		assert_eq!(updated.mime_type, "text/plain");
	}
}
