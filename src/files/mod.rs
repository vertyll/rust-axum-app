//! Files bounded context: upload, storage and metadata of files.
//!
//! Depends on `identity` only through its public auth guards; identity
//! never depends on `files`.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::FilesService;
pub use domain::{FileId, StorageKind};
