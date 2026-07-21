use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The closed set of roles known to the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleName {
	Admin,
	Manager,
	User,
}

impl RoleName {
	pub const fn as_str(self) -> &'static str {
		match self {
			RoleName::Admin => "admin",
			RoleName::Manager => "manager",
			RoleName::User => "user",
		}
	}

	pub const fn description(self) -> &'static str {
		match self {
			RoleName::Admin => "Administrator with full access",
			RoleName::Manager => "User with management privileges",
			RoleName::User => "Regular user with limited access",
		}
	}

	pub const ALL: [RoleName; 3] = [RoleName::Admin, RoleName::Manager, RoleName::User];
}

impl FromStr for RoleName {
	type Err = ();

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value.to_ascii_lowercase().as_str() {
			"admin" => Ok(RoleName::Admin),
			"manager" => Ok(RoleName::Manager),
			"user" => Ok(RoleName::User),
			_ => Err(()),
		}
	}
}

impl fmt::Display for RoleName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.as_str())
	}
}
#[cfg(test)]
mod tests {
	use super::RoleName;

	#[test]
	fn as_str_and_from_str_round_trip() {
		for role in RoleName::ALL {
			assert_eq!(role.as_str().parse::<RoleName>(), Ok(role));
		}
		assert_eq!("ADMIN".parse::<RoleName>(), Ok(RoleName::Admin));
		assert!("root".parse::<RoleName>().is_err());
	}
}
