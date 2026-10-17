use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FilesError {
    #[error("file not found")]
    NotFound,
    #[error("no file uploaded")]
    NoFileUploaded,
    #[error("file larger than {max} bytes")]
    TooLarge { max: u64 },
    #[error("the upload is not pending")]
    NotPending,
    #[error("the storage holds no object of the announced size")]
    NotUploaded,
    #[error("the link is invalid or has expired")]
    InvalidLink,
    #[error("invalid storage type")]
    InvalidStorageType,
    #[error("persistence failure: {0}")]
    PersistenceFailure(String),
    #[error("stored data is corrupt: {0}")]
    CorruptData(String),
    #[error("storage failure: {0}")]
    StorageFailure(String),
}
