use super::file_record::FileRecord;
use crate::files::domain::{FileId, FileRepository, FilesError, NewStoredFile, StoredFile};

#[derive(Clone)]
pub struct ToastyFileRepository {
	db: toasty::Db,
}

impl ToastyFileRepository {
	pub fn new(db: toasty::Db) -> Self {
		Self { db }
	}
}

impl FileRepository for ToastyFileRepository {
	async fn create(&self, file: NewStoredFile) -> Result<StoredFile, FilesError> {
		let mut db = self.db.clone();

		let record = toasty::create!(FileRecord {
			filename: file.filename,
			original_name: file.original_name,
			file_path: file.file_path,
			mime_type: file.mime_type,
			encoding: file.encoding,
			size: file.size,
			storage_type: file.storage.as_str(),
			url: file.url,
			metadata: file.metadata,
		})
		.exec(&mut db)
		.await
		.map_err(persistence)?;

		record.to_domain()
	}

	async fn find_active_by_id(&self, id: FileId) -> Result<Option<StoredFile>, FilesError> {
		let mut db = self.db.clone();

		let record = FileRecord::filter_by_id(id.0)
			.filter(FileRecord::fields().deleted_at().is_none())
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		record.map(|record| record.to_domain()).transpose()
	}

	async fn list_active(&self) -> Result<Vec<StoredFile>, FilesError> {
		let mut db = self.db.clone();

		let records = FileRecord::filter(FileRecord::fields().deleted_at().is_none())
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		records.iter().map(FileRecord::to_domain).collect()
	}

	async fn update(&self, file: &StoredFile) -> Result<(), FilesError> {
		let mut db = self.db.clone();

		toasty::update!(FileRecord::filter_by_id(file.id.0) {
			original_name: file.original_name.clone(),
			mime_type: file.mime_type.clone(),
			encoding: file.encoding.clone(),
			size: file.size,
			storage_type: file.storage.as_str(),
			url: file.url.clone(),
			metadata: file.metadata.clone(),
			deleted_at: file.deleted_at,
			deleted_by_user_id: file.deleted_by_user_id,
		})
		.exec(&mut db)
		.await
		.map_err(persistence)?;

		Ok(())
	}

	async fn delete(&self, id: FileId) -> Result<(), FilesError> {
		let mut db = self.db.clone();

		FileRecord::filter_by_id(id.0)
			.delete()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(())
	}
}

fn persistence(err: toasty::Error) -> FilesError {
	FilesError::PersistenceFailure(err.to_string())
}
