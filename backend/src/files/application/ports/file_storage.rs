use jiff::Timestamp;

use crate::files::domain::{FilesError, StorageKind};

/// Where the bytes of files live. The browser moves the bytes itself, through
/// short-lived signed links this port hands out; the application only decides
/// whether a transfer may happen and checks afterwards what arrived.
pub trait FileStorage: Clone + Send + Sync + 'static {
    /// The storage new uploads go to.
    fn uploads_to(&self) -> StorageKind;

    /// A link the browser sends the bytes of `upload` to, valid only for that object, type and size.
    fn upload_link(
        &self,
        storage: StorageKind,
        upload: &ObjectUpload,
    ) -> impl Future<Output = Result<SignedLink, FilesError>> + Send;

    /// A link the browser downloads the object from, under its original name.
    fn download_link(
        &self,
        storage: StorageKind,
        download: &ObjectDownload,
    ) -> impl Future<Output = Result<SignedLink, FilesError>> + Send;

    /// The size of the stored object, or `None` when nothing was stored under the key.
    fn object_size(
        &self,
        storage: StorageKind,
        key: &str,
    ) -> impl Future<Output = Result<Option<u64>, FilesError>> + Send;

    /// Removes the object; one that is already gone is no error.
    fn remove(&self, storage: StorageKind, key: &str) -> impl Future<Output = Result<(), FilesError>> + Send;
}

/// The object an upload link allows.
#[derive(Debug, Clone)]
pub struct ObjectUpload {
    pub key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size: u64,
}

/// The object a download link serves, and the name and type it is saved under.
#[derive(Debug, Clone)]
pub struct ObjectDownload {
    pub key: String,
    pub file_name: String,
    pub mime_type: String,
}

/// A request the browser makes on its own: the method, the address and the
/// headers it has to send with it, valid until `expires_at`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedLink {
    pub method: &'static str,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub expires_at: Timestamp,
}
