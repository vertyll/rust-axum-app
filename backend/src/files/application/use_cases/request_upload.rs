use std::path::Path;

use uuid::Uuid;

use crate::files::application::commands::RequestUpload;
use crate::files::application::ports::{FileStorage, ObjectUpload, SignedLink};
use crate::files::application::service::FilesService;
use crate::files::domain::{FileRepository, FilesError, NewStoredFile, StoredFile};

impl<R: FileRepository, S: FileStorage> FilesService<R, S> {
    /// Allows an upload: records the file as pending, then hands out the link the
    /// browser sends the bytes to. The row comes first, so nothing can reach the
    /// storage that this application did not allow.
    pub async fn request_upload(&self, request: RequestUpload) -> Result<(StoredFile, SignedLink), FilesError> {
        if request.size == 0 {
            return Err(FilesError::NoFileUploaded);
        }
        if request.size > self.max_size {
            return Err(FilesError::TooLarge { max: self.max_size });
        }
        let size = i64::try_from(request.size).map_err(|_| FilesError::TooLarge { max: self.max_size })?;
        let storage = self.storage.uploads_to();
        let file = self
            .repository
            .create(NewStoredFile {
                object_key: object_key(&request.original_name),
                original_name: request.original_name,
                mime_type: request.mime_type,
                size,
                storage,
            })
            .await?;

        let upload = ObjectUpload {
            key: file.object_key.clone(),
            file_name: file.original_name.clone(),
            mime_type: file.mime_type.clone(),
            size: request.size,
        };
        match self.storage.upload_link(storage, &upload).await {
            Ok(link) => Ok((file, link)),
            Err(err) => {
                if let Err(cleanup) = self.repository.delete(file.id).await {
                    tracing::error!(file = %file.id, "could not remove a pending upload without a link: {cleanup}");
                }
                Err(err)
            }
        }
    }
}

/// A new name for the object, keeping only a plain extension of the original, so
/// two uploads never collide and nothing the client sent becomes part of a path.
fn object_key(original_name: &str) -> String {
    let id = Uuid::new_v4();
    match Path::new(original_name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some(extension) if is_plain(extension) => format!("{id}.{}", extension.to_ascii_lowercase()),
        _ => id.to_string(),
    }
}

fn is_plain(extension: &str) -> bool {
    (1..=10).contains(&extension.len()) && extension.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::object_key;
    use crate::files::application::testing::{files_harness, request};
    use crate::files::domain::{FileStatus, FilesError, StorageKind};

    #[tokio::test]
    async fn an_allowed_upload_is_pending_until_the_bytes_arrive() {
        let (service, repository, _) = files_harness();

        let (file, link) = service.request_upload(request("note.txt", 5)).await.unwrap();

        assert_eq!(file.status, FileStatus::Pending);
        assert_eq!(file.storage, StorageKind::S3);
        assert_eq!(link.method, "PUT");
        assert!(repository.get_any(file.id).is_some());
    }

    #[tokio::test]
    async fn an_empty_upload_is_refused() {
        let (service, _, _) = files_harness();

        let err = service.request_upload(request("empty.txt", 0)).await.unwrap_err();

        assert_eq!(err, FilesError::NoFileUploaded);
    }

    #[tokio::test]
    async fn an_upload_over_the_limit_is_refused() {
        let (service, _, _) = files_harness();

        let err = service.request_upload(request("big.bin", 101)).await.unwrap_err();

        assert_eq!(err, FilesError::TooLarge { max: 100 });
    }

    #[test]
    fn the_object_keeps_only_a_plain_extension() {
        assert_eq!(
            object_key("Report.PDF")
                .rsplit_once('.')
                .map(|(_, extension)| extension),
            Some("pdf")
        );
        assert!(!object_key("../../etc/passwd").contains('/'));
        assert!(!object_key("photo.j p g").contains(' '));
    }
}
