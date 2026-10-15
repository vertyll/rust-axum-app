use jiff::Timestamp;
use toasty::Deferred;

use super::{PermissionRecord, RoleRecord};

/// A role granting a permission.
#[derive(Debug, toasty::Model)]
#[table = "role_permissions"]
pub struct RolePermissionRecord {
    #[key]
    #[auto]
    pub id: i64,
    #[index]
    pub role_id: i64,
    #[index]
    pub permission_id: i64,
    #[auto]
    pub created_at: Timestamp,
    #[auto]
    pub updated_at: Timestamp,

    #[belongs_to(key = role_id, references = id)]
    pub role_record: Deferred<RoleRecord>,
    #[belongs_to(key = permission_id, references = id)]
    pub permission_record: Deferred<PermissionRecord>,
}
