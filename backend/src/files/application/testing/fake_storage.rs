use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use jiff::Timestamp;

use crate::files::application::ports::{FileStorage, ObjectDownload, ObjectUpload, SignedLink};
use crate::files::domain::{FilesError, StorageKind};

/// A storage in memory: the objects the browser would have sent, by key and size.
#[derive(Clone, Default)]
pub(crate) struct FakeStorage {
    objects: Arc<Mutex<HashMap<String, u64>>>,
    removed: Arc<Mutex<Vec<String>>>,
}

impl FakeStorage {
    /// Stores an object, as the browser does with an upload link.
    pub fn put(&self, key: &str, size: u64) {
        self.objects.lock().unwrap().insert(key.to_string(), size);
    }

    pub fn removed(&self) -> Vec<String> {
        self.removed.lock().unwrap().clone()
    }
}

impl FileStorage for FakeStorage {
    fn uploads_to(&self) -> StorageKind {
        StorageKind::S3
    }

    async fn upload_link(&self, _storage: StorageKind, upload: &ObjectUpload) -> Result<SignedLink, FilesError> {
        Ok(SignedLink {
            method: "PUT",
            url: format!("https://storage.test/{}", upload.key),
            headers: vec![("content-type".into(), upload.mime_type.clone())],
            expires_at: Timestamp::now(),
        })
    }

    async fn download_link(&self, _storage: StorageKind, download: &ObjectDownload) -> Result<SignedLink, FilesError> {
        Ok(SignedLink {
            method: "GET",
            url: format!("https://storage.test/{}", download.key),
            headers: Vec::new(),
            expires_at: Timestamp::now(),
        })
    }

    async fn object_size(&self, _storage: StorageKind, key: &str) -> Result<Option<u64>, FilesError> {
        Ok(self.objects.lock().unwrap().get(key).copied())
    }

    async fn remove(&self, _storage: StorageKind, key: &str) -> Result<(), FilesError> {
        self.objects.lock().unwrap().remove(key);
        self.removed.lock().unwrap().push(key.to_string());
        Ok(())
    }
}
