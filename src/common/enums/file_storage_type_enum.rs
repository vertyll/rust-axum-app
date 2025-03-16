use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileStorageTypeEnum {
	Local,
}

impl FileStorageTypeEnum {
	pub fn as_str(&self) -> &'static str {
		match self {
			FileStorageTypeEnum::Local => "local",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"local" => Some(FileStorageTypeEnum::Local),
			_ => None,
		}
	}
}

impl Display for FileStorageTypeEnum {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
