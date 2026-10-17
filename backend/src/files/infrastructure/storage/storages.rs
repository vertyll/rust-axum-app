use super::{LocalFileStorage, S3FileStorage};
use crate::files::application::{FileStorage, ObjectDownload, ObjectUpload, SignedLink};
use crate::files::domain::{FilesError, StorageKind};

/// Both storages behind one port: new uploads go to the one configured, and every
/// other operation to the one that holds the file, so changing the storage leaves
/// the files already stored where they are and still readable.
#[derive(Clone)]
pub struct Storages {
    uploads_to: StorageKind,
    local: LocalFileStorage,
    s3: S3FileStorage,
}

impl Storages {
    #[must_use]
    pub fn new(uploads_to: StorageKind, local: LocalFileStorage, s3: S3FileStorage) -> Self {
        Self { uploads_to, local, s3 }
    }
}

impl FileStorage for Storages {
    fn uploads_to(&self) -> StorageKind {
        self.uploads_to
    }

    async fn upload_link(&self, storage: StorageKind, upload: &ObjectUpload) -> Result<SignedLink, FilesError> {
        match storage {
            StorageKind::Local => self.local.upload_link(upload),
            StorageKind::S3 => self.s3.upload_link(upload).await,
        }
    }

    async fn download_link(&self, storage: StorageKind, download: &ObjectDownload) -> Result<SignedLink, FilesError> {
        match storage {
            StorageKind::Local => self.local.download_link(download),
            StorageKind::S3 => self.s3.download_link(download).await,
        }
    }

    async fn object_size(&self, storage: StorageKind, key: &str) -> Result<Option<u64>, FilesError> {
        match storage {
            StorageKind::Local => self.local.object_size(key).await,
            StorageKind::S3 => self.s3.object_size(key).await,
        }
    }

    async fn remove(&self, storage: StorageKind, key: &str) -> Result<(), FilesError> {
        match storage {
            StorageKind::Local => self.local.remove(key).await,
            StorageKind::S3 => self.s3.remove(key).await,
        }
    }
}
