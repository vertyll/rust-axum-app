use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// The realm roles the application understands; Keycloak names them in capitals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleName {
	Admin,
	User,
}

impl RoleName {
	pub const fn as_str(self) -> &'static str {
		match self {
			RoleName::Admin => "admin",
			RoleName::User => "user",
		}
	}

	pub const fn description(self) -> &'static str {
		match self {
			RoleName::Admin => "Administrator",
			RoleName::User => "Signed-in user",
		}
	}

	pub const ALL: [RoleName; 2] = [RoleName::Admin, RoleName::User];
}

impl FromStr for RoleName {
	type Err = ();

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value.to_ascii_lowercase().as_str() {
			"admin" => Ok(RoleName::Admin),
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
	fn realm_role_names_parse_regardless_of_case() {
		for role in RoleName::ALL {
			assert_eq!(role.as_str().parse::<RoleName>(), Ok(role));
		}
		assert_eq!("ADMIN".parse::<RoleName>(), Ok(RoleName::Admin));
		assert!("offline_access".parse::<RoleName>().is_err());
	}
}
