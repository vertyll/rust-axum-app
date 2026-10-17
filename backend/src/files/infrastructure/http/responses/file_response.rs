use jiff::Timestamp;
use serde::Serialize;

use crate::files::domain::{StorageKind, StoredFile};

/// A file's description. Where its bytes are is never part of it: a client
/// exchanges the id for a link at the moment it needs one.
#[derive(Debug, Serialize)]
pub struct FileResponse {
    pub id: i64,
    pub original_name: String,
    pub mime_type: String,
    pub encoding: String,
    pub size: i64,
    pub storage_type: StorageKind,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl From<StoredFile> for FileResponse {
    fn from(file: StoredFile) -> Self {
        Self {
            id: file.id.0,
            original_name: file.original_name,
            mime_type: file.mime_type,
            encoding: file.encoding,
            size: file.size,
            storage_type: file.storage,
            created_at: file.created_at,
            updated_at: file.updated_at,
        }
    }
}
