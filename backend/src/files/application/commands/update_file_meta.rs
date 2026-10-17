/// The descriptive fields of a file. Where the bytes are, and how many, belong
/// to the upload and are not changed afterwards.
#[derive(Debug, Default)]
pub struct UpdateFileMeta {
    pub original_name: Option<String>,
    pub mime_type: Option<String>,
    pub encoding: Option<String>,
}
