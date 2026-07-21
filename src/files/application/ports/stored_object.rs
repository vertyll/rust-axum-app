#[derive(Debug, Clone)]
pub struct StoredObject {
	pub filename: String,
	pub file_path: String,
	pub url: String,
	pub metadata: serde_json::Value,
}
