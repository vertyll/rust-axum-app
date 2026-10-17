use serde::Deserialize;
use validator::Validate;

use crate::files::application::commands::RequestUpload;

/// The upload the browser asks to make.
#[derive(Debug, Deserialize, Validate)]
pub struct UploadTicketRequest {
    #[validate(length(min = 1, max = 255, message = "files.validators.file.name.length"))]
    pub original_name: String,
    #[validate(length(min = 1, max = 255, message = "files.validators.file.mime_type.length"))]
    pub mime_type: String,
    pub size: u64,
}

impl From<UploadTicketRequest> for RequestUpload {
    fn from(request: UploadTicketRequest) -> Self {
        Self {
            original_name: request.original_name,
            mime_type: request.mime_type,
            size: request.size,
        }
    }
}
