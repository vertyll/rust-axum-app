use std::collections::BTreeSet;

use serde::Deserialize;
use validator::Validate;

use crate::identity::application::{NewRole, RoleChanges};

/// `POST /roles`. The name is checked by the domain, as Keycloak takes it.
#[derive(Debug, Deserialize, Validate)]
pub struct NewRoleRequest {
    pub name: String,
    #[serde(default)]
    #[validate(length(max = 255, message = "roles.validators.description.length"))]
    pub description: String,
    #[serde(default)]
    pub unrestricted: bool,
    #[serde(default)]
    pub permissions: BTreeSet<String>,
}

impl From<NewRoleRequest> for NewRole {
    fn from(request: NewRoleRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            unrestricted: request.unrestricted,
            permissions: request.permissions,
        }
    }
}

/// `PUT /roles/{name}`: what the role becomes, every field given.
#[derive(Debug, Deserialize, Validate)]
pub struct RoleChangesRequest {
    #[validate(length(max = 255, message = "roles.validators.description.length"))]
    pub description: String,
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
}

impl From<RoleChangesRequest> for RoleChanges {
    fn from(request: RoleChangesRequest) -> Self {
        Self {
            description: request.description,
            unrestricted: request.unrestricted,
            permissions: request.permissions,
        }
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::NewRoleRequest;

    #[test]
    fn a_role_takes_only_its_name_and_a_short_description() {
        let request: NewRoleRequest = serde_json::from_str(r#"{"name":"editor"}"#).unwrap();
        assert!(request.validate().is_ok());
        assert!(!request.unrestricted && request.permissions.is_empty());

        let long = format!(r#"{{"name":"editor","description":"{}"}}"#, "a".repeat(256));
        let request: NewRoleRequest = serde_json::from_str(&long).unwrap();
        assert!(request.validate().is_err());
    }
}
