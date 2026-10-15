use jiff::Timestamp;
use toasty::Deferred;

use super::{RolePermissionRecord, UserRoleRecord};

#[derive(Debug, toasty::Model)]
#[table = "roles"]
pub struct RoleRecord {
    #[key]
    #[auto]
    pub id: i64,
    #[unique]
    pub name: String,
    pub description: Option<String>,
    pub unrestricted: bool,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,

    #[has_many]
    pub user_roles: Deferred<Vec<UserRoleRecord>>,
    #[has_many]
    pub role_permissions: Deferred<Vec<RolePermissionRecord>>,
}
