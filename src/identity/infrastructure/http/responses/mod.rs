use jiff::Timestamp;
use serde::Serialize;

use crate::identity::application::AuthSession;
use crate::identity::domain::{Grants, RegisteredPermission, Role, RoleName, User};

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub keycloak_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<RoleName>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id.0,
            keycloak_id: user.keycloak_id.to_string(),
            email: user.email.to_string(),
            first_name: user.first_name,
            last_name: user.last_name,
            roles: user.roles,
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<RoleName>,
}

impl From<&AuthSession> for SessionResponse {
    fn from(session: &AuthSession) -> Self {
        Self {
            user_id: session.identity.keycloak_id.to_string(),
            email: session.identity.email.to_string(),
            roles: session.identity.roles.clone(),
        }
    }
}

/// A role and the permissions it grants.
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub name: RoleName,
    pub description: String,
    pub unrestricted: bool,
    pub permissions: Vec<String>,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl From<Role> for RoleResponse {
    fn from(role: Role) -> Self {
        Self {
            name: role.name,
            description: role.description,
            unrestricted: role.unrestricted,
            permissions: role.permissions.into_iter().collect(),
            created_at: role.created_at,
            updated_at: role.updated_at,
        }
    }
}

/// One module and the permissions it declares.
#[derive(Debug, Serialize)]
pub struct ModuleResponse {
    pub module: String,
    pub permissions: Vec<PermissionResponse>,
}

#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub name: String,
    pub description: String,
}

impl ModuleResponse {
    /// The permissions grouped by the module that declares them; `permissions`
    /// comes sorted by module, then name.
    #[must_use]
    pub fn grouped(permissions: Vec<RegisteredPermission>) -> Vec<Self> {
        let mut modules: Vec<Self> = Vec::new();
        for permission in permissions {
            let entry = PermissionResponse {
                name: permission.name,
                description: permission.description,
            };
            match modules.last_mut() {
                Some(module) if module.module == permission.module => module.permissions.push(entry),
                _ => modules.push(Self {
                    module: permission.module,
                    permissions: vec![entry],
                }),
            }
        }
        modules
    }
}

/// What the caller's roles grant: every permission when `unrestricted`.
#[derive(Debug, Serialize)]
pub struct GrantsResponse {
    pub unrestricted: bool,
    pub permissions: Vec<String>,
}

impl From<&Grants> for GrantsResponse {
    fn from(grants: &Grants) -> Self {
        Self {
            unrestricted: grants.is_unrestricted(),
            permissions: grants.permissions().iter().cloned().collect(),
        }
    }
}
