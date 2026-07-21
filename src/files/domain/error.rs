use thiserror::Error;

#[derive(Debug, Error)]
pub enum FilesError {
	#[error("file not found")]
	NotFound,
	#[error("no file uploaded")]
	NoFileUploaded,
	#[error("file upload failed")]
	UploadFailed,
	#[error("invalid storage type")]
	InvalidStorageType,
	#[error("persistence failure: {0}")]
	PersistenceFailure(String),
	#[error("stored data is corrupt: {0}")]
	CorruptData(String),
	#[error("storage failure: {0}")]
	StorageFailure(String),
}
