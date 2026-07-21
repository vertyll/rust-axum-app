//! Files bounded context: upload, storage and metadata of files.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::FilesService;
pub use domain::{FileId, StorageKind};
