//! `PermissionRegistry` — the permissions the modules declare, as they were
//! registered at the last start.

use super::error::IdentityError;
use super::permission::RegisteredPermission;

pub trait PermissionRegistry: Clone + Send + Sync + 'static {
    /// Every permission registered, sorted by module and name.
    fn list(&self) -> impl Future<Output = Result<Vec<RegisteredPermission>, IdentityError>> + Send;

    /// Makes the registry exactly `declared`, atomically: new permissions are
    /// added, the others updated, and those no module declares any more are
    /// removed. The roles are not touched: withdrawing a permission from them
    /// is the use case's, through `RoleRepository`.
    fn register(&self, declared: &[RegisteredPermission]) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
