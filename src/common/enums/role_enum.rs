use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleEnum {
	Admin,
	Manager,
	User,
}

impl RoleEnum {
	pub fn as_str(&self) -> &'static str {
		match self {
			RoleEnum::Admin => "admin",
			RoleEnum::Manager => "manager",
			RoleEnum::User => "user",
		}
	}

	pub fn from_str(s: &str) -> Option<Self> {
		match s.to_lowercase().as_str() {
			"admin" => Some(RoleEnum::Admin),
			"manager" => Some(RoleEnum::Manager),
			"user" => Some(RoleEnum::User),
			_ => None,
		}
	}
}

impl Display for RoleEnum {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.as_str())
	}
}
