//! Roles: the realm roles Keycloak puts in a token, and what each of them
//! grants in this application.

use std::collections::BTreeSet;
use std::fmt;
use std::str::FromStr;

use jiff::Timestamp;
use serde::{Deserialize, Serialize};

use super::error::IdentityError;

/// The name of a role: the realm role Keycloak puts in a token, in lower case.
/// Keycloak names realm roles in capitals, so `ADMIN` there is `admin` here.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RoleName(String);

impl RoleName {
    /// The role that holds every permission, as shipped.
    pub const ADMIN: &str = "admin";
    /// The role every signed-in account holds, as shipped.
    pub const USER: &str = "user";

    const MAX_LEN: usize = 64;

    /// A name of lower-case letters, digits, `_` and `-`, starting with a
    /// letter; the case of `value` does not matter.
    pub fn parse(value: &str) -> Result<Self, IdentityError> {
        let name = value.trim().to_ascii_lowercase();
        let valid = name.len() <= Self::MAX_LEN
            && name.starts_with(|c: char| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-');
        if valid {
            Ok(Self(name))
        } else {
            Err(IdentityError::InvalidRoleName)
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether this is one of the roles the application cannot do without, `admin` or `user`, which are never
    /// removed.
    #[must_use]
    pub fn is_system(&self) -> bool {
        self.0 == Self::ADMIN || self.0 == Self::USER
    }

    /// The realm role's name in Keycloak.
    #[must_use]
    pub fn realm_role(&self) -> String {
        self.0.to_ascii_uppercase()
    }
}

impl FromStr for RoleName {
    type Err = IdentityError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<String> for RoleName {
    type Error = IdentityError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<RoleName> for String {
    fn from(name: RoleName) -> Self {
        name.0
    }
}

impl fmt::Display for RoleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A role and what it grants. An `unrestricted` role holds every permission,
/// also one a module declares after the role was made, so it is a flag rather
/// than the list of every permission today.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Role {
    pub name: RoleName,
    pub description: String,
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// What a caller may do: the union of what their roles grant.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grants {
    unrestricted: bool,
    permissions: BTreeSet<String>,
}

impl Grants {
    #[must_use]
    pub fn of<'a>(roles: impl IntoIterator<Item = &'a Role>) -> Self {
        roles.into_iter().fold(Self::default(), |mut grants, role| {
            grants.unrestricted |= role.unrestricted;
            grants.permissions.extend(role.permissions.iter().cloned());
            grants
        })
    }

    #[must_use]
    pub fn allows(&self, permission: &str) -> bool {
        self.unrestricted || self.permissions.contains(permission)
    }

    #[must_use]
    pub fn is_unrestricted(&self) -> bool {
        self.unrestricted
    }

    /// The permissions granted by name; an unrestricted caller holds more.
    #[must_use]
    pub fn permissions(&self) -> &BTreeSet<String> {
        &self.permissions
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use jiff::Timestamp;

    use super::{Grants, Role, RoleName};

    fn role(name: &str, unrestricted: bool, permissions: &[&str]) -> Role {
        Role {
            name: RoleName::parse(name).unwrap(),
            description: String::new(),
            unrestricted,
            permissions: permissions.iter().map(ToString::to_string).collect::<BTreeSet<_>>(),
            created_at: Timestamp::UNIX_EPOCH,
            updated_at: Timestamp::UNIX_EPOCH,
        }
    }

    #[test]
    fn names_are_lower_case_and_keycloak_names_them_in_capitals() {
        let name = RoleName::parse(" Editor_2 ").unwrap();
        assert_eq!(name.as_str(), "editor_2");
        assert_eq!(name.realm_role(), "EDITOR_2");
        assert_eq!("ADMIN".parse::<RoleName>().unwrap().as_str(), RoleName::ADMIN);
    }

    #[test]
    fn a_name_starts_with_a_letter_and_holds_no_spaces() {
        for invalid in ["", "2nd", "two words", "zażółć", &"a".repeat(65)] {
            assert!(RoleName::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn grants_are_the_union_of_the_roles() {
        let grants = Grants::of(&[role("a", false, &["X"]), role("b", false, &["Y"])]);
        assert!(grants.allows("X") && grants.allows("Y"));
        assert!(!grants.allows("Z"));
        assert!(!grants.is_unrestricted());
    }

    #[test]
    fn an_unrestricted_role_allows_what_no_role_lists() {
        let grants = Grants::of(&[role("admin", true, &[])]);
        assert!(grants.allows("DECLARED_TOMORROW"));
    }
}
