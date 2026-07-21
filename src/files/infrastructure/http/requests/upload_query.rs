use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
	pub storage_type: Option<String>,
}
