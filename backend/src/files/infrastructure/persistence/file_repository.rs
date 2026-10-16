use super::records::FileRecord;
use crate::files::domain::{FileId, FileRepository, FilesError, NewStoredFile, StoredFile};
use crate::shared_kernel::pagination::{PageRequest, PageResult};

#[derive(Clone)]
pub struct ToastyFileRepository {
    db: toasty::Db,
}

impl ToastyFileRepository {
    #[must_use]
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

    async fn page_active(&self, request: PageRequest) -> Result<PageResult<StoredFile>, FilesError> {
        let mut db = self.db.clone();
        let active = || FileRecord::filter(FileRecord::fields().deleted_at().is_none());

        let total = active().count().exec(&mut db).await.map_err(persistence)?;
        let records = active()
            .order_by(FileRecord::fields().id().desc())
            .limit(request.limit())
            .offset(request.offset())
            .exec(&mut db)
            .await
            .map_err(persistence)?;

        let items = records.iter().map(FileRecord::to_domain).collect::<Result<_, _>>()?;
        Ok(PageResult::new(items, request, total))
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

#[expect(
    clippy::needless_pass_by_value,
    reason = "passed to `map_err`, which hands over the error it maps"
)]
fn persistence(err: toasty::Error) -> FilesError {
    FilesError::PersistenceFailure(err.to_string())
}
