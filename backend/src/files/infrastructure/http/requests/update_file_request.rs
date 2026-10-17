use serde::Deserialize;
use validator::Validate;

use crate::files::application::commands::UpdateFileMeta;

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateFileRequest {
    #[validate(length(min = 1, max = 255, message = "files.validators.file.name.length"))]
    pub original_name: Option<String>,
    #[validate(length(min = 1, max = 255, message = "files.validators.file.mime_type.length"))]
    pub mime_type: Option<String>,
    pub encoding: Option<String>,
}

impl From<UpdateFileRequest> for UpdateFileMeta {
    fn from(request: UpdateFileRequest) -> Self {
        Self {
            original_name: request.original_name,
            mime_type: request.mime_type,
            encoding: request.encoding,
        }
    }
}
