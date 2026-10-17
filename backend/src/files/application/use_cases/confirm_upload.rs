use crate::files::application::ports::FileStorage;
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    /// Confirms an upload against the storage: the object has to exist at the size
    /// the upload was allowed for. An object of another size is removed, as it is
    /// not the file this application allowed.
    pub async fn confirm_upload(&self, id: FileId) -> Result<StoredFile, FilesError> {
        let mut file = self
            .repository
            .find_active_by_id(id)
            .await?
            .ok_or(FilesError::NotFound)?;
        let stored = self
            .storage
            .object_size(file.storage, &file.object_key)
            .await?
            .ok_or(FilesError::NotUploaded)?;
        if let Err(err) = file.confirm(stored) {
            if err == FilesError::NotUploaded {
                self.storage.remove(file.storage, &file.object_key).await?;
            }
            return Err(err);
        }
        self.repository.update(&file).await?;
        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use crate::files::application::testing::{files_harness, request};
    use crate::files::domain::{FileStatus, FilesError};

    #[tokio::test]
    async fn an_object_of_the_announced_size_confirms_the_upload() {
        let (service, _, storage) = files_harness();
        let (file, _) = service.request_upload(request("note.txt", 5)).await.unwrap();
        storage.put(&file.object_key, 5);

        let confirmed = service.confirm_upload(file.id).await.unwrap();

        assert_eq!(confirmed.status, FileStatus::Uploaded);
    }

    #[tokio::test]
    async fn nothing_stored_is_not_an_upload() {
        let (service, _, _) = files_harness();
        let (file, _) = service.request_upload(request("note.txt", 5)).await.unwrap();

        let err = service.confirm_upload(file.id).await.unwrap_err();

        assert_eq!(err, FilesError::NotUploaded);
    }

    #[tokio::test]
    async fn an_object_of_another_size_is_refused_and_removed() {
        let (service, _, storage) = files_harness();
        let (file, _) = service.request_upload(request("note.txt", 5)).await.unwrap();
        storage.put(&file.object_key, 9);

        let err = service.confirm_upload(file.id).await.unwrap_err();

        assert_eq!(err, FilesError::NotUploaded);
        assert_eq!(storage.removed(), vec![file.object_key]);
    }
}
