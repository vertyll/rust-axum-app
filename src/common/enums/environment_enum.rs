use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvironmentEnum {
	Development,
}

impl EnvironmentEnum {
	pub fn as_str(&self) -> &'static str {
		match self {
			EnvironmentEnum::Development => "development",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"development" => Some(EnvironmentEnum::Development),
			_ => None,
		}
	}
}

impl Display for EnvironmentEnum {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
