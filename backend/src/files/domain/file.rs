use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use super::error::FilesError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub i64);

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Where a file's bytes live. Each file keeps the storage it was uploaded to, so
/// changing the storage for new uploads leaves the older files readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorageKind {
    Local,
    S3,
}

impl StorageKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            StorageKind::Local => "local",
            StorageKind::S3 => "s3",
        }
    }
}

impl FromStr for StorageKind {
    type Err = FilesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(StorageKind::Local),
            "s3" => Ok(StorageKind::S3),
            _ => Err(FilesError::InvalidStorageType),
        }
    }
}

impl fmt::Display for StorageKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Whether the bytes have arrived. A file is `Pending` from the moment an upload is
/// allowed until the storage confirms the object, and only an `Uploaded` file is
/// listed, read or downloaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStatus {
    Pending,
    Uploaded,
}

impl FileStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            FileStatus::Pending => "pending",
            FileStatus::Uploaded => "uploaded",
        }
    }
}

impl FromStr for FileStatus {
    type Err = FilesError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(FileStatus::Pending),
            "uploaded" => Ok(FileStatus::Uploaded),
            _ => Err(FilesError::CorruptData(format!("unknown file status {value}"))),
        }
    }
}

#[derive(Debug, Clone)]
pub struct StoredFile {
    pub id: FileId,
    /// The object's name in its storage, never derived from what the client sent.
    pub object_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub encoding: String,
    pub size: i64,
    pub storage: StorageKind,
    pub status: FileStatus,
    pub deleted_at: Option<Timestamp>,
    /// Cross-context reference by id only — no ORM relation into the
    /// identity module, so the module boundary stays real.
    pub deleted_by_user_id: Option<i64>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl StoredFile {
    #[must_use]
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn soft_delete(&mut self, by_user_id: i64, now: Timestamp) {
        self.deleted_at = Some(now);
        self.deleted_by_user_id = Some(by_user_id);
    }

    /// Records that the bytes arrived, which only an upload still pending and only
    /// the size it announced can do: the storage's word on the size is the evidence,
    /// the client saying "done" is not.
    ///
    /// # Errors
    ///
    /// `FilesError::NotPending` once confirmed, `FilesError::NotUploaded` when the
    /// stored object is not the size the upload was allowed for.
    pub fn confirm(&mut self, stored_size: u64) -> Result<(), FilesError> {
        if self.status != FileStatus::Pending {
            return Err(FilesError::NotPending);
        }
        if i64::try_from(stored_size).ok() != Some(self.size) {
            return Err(FilesError::NotUploaded);
        }
        self.status = FileStatus::Uploaded;
        Ok(())
    }
}

/// An upload allowed but not yet made: the row exists before the bytes, so nothing
/// can reach the storage that this application did not allow.
#[derive(Debug, Clone)]
pub struct NewStoredFile {
    pub object_key: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: i64,
    pub storage: StorageKind,
}
#[cfg(test)]
mod tests {
    use jiff::Timestamp;

    use super::*;

    fn pending(size: i64) -> StoredFile {
        StoredFile {
            id: FileId(1),
            object_key: "0192a4f0.txt".into(),
            original_name: "note.txt".into(),
            mime_type: "text/plain".into(),
            encoding: "binary".into(),
            size,
            storage: StorageKind::S3,
            status: FileStatus::Pending,
            deleted_at: None,
            deleted_by_user_id: None,
            created_at: Timestamp::UNIX_EPOCH,
            updated_at: Timestamp::UNIX_EPOCH,
        }
    }

    #[test]
    fn storage_kind_parses_case_insensitively() {
        assert_eq!("Local".parse::<StorageKind>(), Ok(StorageKind::Local));
        assert_eq!("S3".parse::<StorageKind>(), Ok(StorageKind::S3));
        assert!(matches!(
            "ftp".parse::<StorageKind>(),
            Err(FilesError::InvalidStorageType)
        ));
    }

    #[test]
    fn soft_delete_marks_who_and_when() {
        let mut file = pending(5);
        assert!(!file.is_deleted());

        file.soft_delete(42, Timestamp::now());

        assert!(file.is_deleted());
        assert_eq!(file.deleted_by_user_id, Some(42));
    }

    #[test]
    fn an_upload_of_the_announced_size_is_confirmed() {
        let mut file = pending(5);

        file.confirm(5).unwrap();

        assert_eq!(file.status, FileStatus::Uploaded);
    }

    #[test]
    fn an_object_of_another_size_is_not_the_upload() {
        let mut file = pending(5);

        assert_eq!(file.confirm(6), Err(FilesError::NotUploaded));
        assert_eq!(file.status, FileStatus::Pending);
    }

    #[test]
    fn a_confirmed_upload_is_not_confirmed_again() {
        let mut file = pending(5);
        file.confirm(5).unwrap();

        assert_eq!(file.confirm(5), Err(FilesError::NotPending));
    }
}
