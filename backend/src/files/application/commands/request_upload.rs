/// An upload the browser asks to make: the name, type and size it announces.
#[derive(Debug)]
pub struct RequestUpload {
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
}
