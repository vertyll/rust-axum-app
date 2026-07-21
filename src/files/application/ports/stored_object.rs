#[derive(Debug, Clone)]
pub struct StoredObject {
	pub filename: String,
	pub path: String,
	pub url: String,
	pub metadata: serde_json::Value,
}
