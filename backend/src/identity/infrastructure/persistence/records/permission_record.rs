use jiff::Timestamp;
use toasty::Deferred;

use super::RolePermissionRecord;

/// A permission some module declares; the registry is rewritten at start-up.
#[derive(Debug, toasty::Model)]
#[table = "permissions"]
pub struct PermissionRecord {
    #[key]
    #[auto]
    pub id: i64,
    #[unique]
    pub name: String,
    pub module: String,
    pub description: String,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,

    #[has_many]
    pub role_permissions: Deferred<Vec<RolePermissionRecord>>,
}
