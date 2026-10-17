use std::collections::BTreeSet;

use serde::Deserialize;
use validator::{Validate, ValidationError};

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

/// The languages Keycloak's pages and this application both have.
pub(crate) const UI_LOCALES: [&str; 2] = ["pl", "en"];

/// `PUT /users/me/language`: a language Keycloak's pages and this application both have.
#[derive(Debug, Deserialize, Validate)]
pub struct AccountLanguageRequest {
    #[validate(custom(function = "supported_language", message = "validation.invalid"))]
    pub language: String,
}

fn supported_language(language: &str) -> Result<(), ValidationError> {
    if UI_LOCALES.contains(&language) {
        Ok(())
    } else {
        Err(ValidationError::new("unsupported_language"))
    }
}

#[cfg(test)]
mod tests {
    use validator::Validate;

    use super::{AccountLanguageRequest, NewRoleRequest};

    #[test]
    fn a_role_takes_only_its_name_and_a_short_description() {
        let request: NewRoleRequest = serde_json::from_str(r#"{"name":"editor"}"#).unwrap();
        assert!(request.validate().is_ok());
        assert!(!request.unrestricted && request.permissions.is_empty());

        let long = format!(r#"{{"name":"editor","description":"{}"}}"#, "a".repeat(256));
        let request: NewRoleRequest = serde_json::from_str(&long).unwrap();
        assert!(request.validate().is_err());
    }

    #[test]
    fn an_account_takes_only_a_language_both_sides_have() {
        let request: AccountLanguageRequest = serde_json::from_str(r#"{"language":"en"}"#).unwrap();
        assert!(request.validate().is_ok());

        let request: AccountLanguageRequest = serde_json::from_str(r#"{"language":"de"}"#).unwrap();
        assert!(request.validate().is_err());
    }
}
