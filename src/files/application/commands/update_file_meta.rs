use crate::files::domain::StorageKind;

#[derive(Debug, Default)]
pub struct UpdateFileMeta {
	pub original_name: Option<String>,
	pub mime_type: Option<String>,
	pub encoding: Option<String>,
	pub size: Option<i64>,
	pub storage: Option<StorageKind>,
	pub url: Option<String>,
}
