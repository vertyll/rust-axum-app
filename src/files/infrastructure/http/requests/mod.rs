//! Inbound request DTOs of the files context, one file per struct.

mod update_file_request;
mod upload_query;

pub use update_file_request::UpdateFileRequest;
pub use upload_query::UploadQuery;
