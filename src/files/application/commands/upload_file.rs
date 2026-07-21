use crate::files::domain::StorageKind;

#[derive(Debug)]
pub struct UploadFile {
	pub data: Vec<u8>,
	pub original_name: String,
	pub mime_type: String,
	pub storage: StorageKind,
}
