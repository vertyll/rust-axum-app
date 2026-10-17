use crate::files::application::ports::{FileStorage, ObjectDownload, SignedLink};
use crate::files::application::service::FilesService;
use crate::files::domain::{FileId, FileRepository, FilesError};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    /// A fresh link to the file's bytes, asked for at the moment of the download,
    /// as a link kept in a page would expire while the page is open.
    pub async fn download_link(&self, id: FileId) -> Result<SignedLink, FilesError> {
        let file = self.get(id).await?;
        let download = ObjectDownload {
            key: file.object_key,
            file_name: file.original_name,
            mime_type: file.mime_type,
        };
        self.storage.download_link(file.storage, &download).await
    }
}

#[cfg(test)]
mod tests {
    use crate::files::application::testing::{files_harness, request, uploaded};
    use crate::files::domain::FilesError;

    #[tokio::test]
    async fn an_uploaded_file_gets_a_link() {
        let (service, _, _) = files_harness();
        let file = uploaded(&service, "note.txt").await;

        let link = service.download_link(file.id).await.unwrap();

        assert_eq!(link.method, "GET");
    }

    #[tokio::test]
    async fn a_pending_upload_has_nothing_to_download() {
        let (service, _, _) = files_harness();
        let (file, _) = service.request_upload(request("note.txt", 5)).await.unwrap();

        let err = service.download_link(file.id).await.unwrap_err();

        assert_eq!(err, FilesError::NotFound);
    }
}
