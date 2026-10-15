//! `RoleRepository` — the roles, what they grant, and the registry of the
//! permissions the modules declare.

use super::error::IdentityError;
use super::permission::RegisteredPermission;
use super::role::{Role, RoleName};

pub trait RoleRepository: Clone + Send + Sync + 'static {
    fn list(&self) -> impl Future<Output = Result<Vec<Role>, IdentityError>> + Send;

    fn find(&self, name: &RoleName) -> impl Future<Output = Result<Option<Role>, IdentityError>> + Send;

    /// The roles of these names that exist; the others are left out.
    fn find_many(&self, names: &[RoleName]) -> impl Future<Output = Result<Vec<Role>, IdentityError>> + Send;

    /// Stores a new role and its grants atomically.
    fn create(&self, role: &Role) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Writes a role and its grants back atomically.
    fn update(&self, role: &Role) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Removes a role, its grants and the accounts' links to it.
    fn delete(&self, name: &RoleName) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Every permission registered, sorted by module and name.
    fn permissions(&self) -> impl Future<Output = Result<Vec<RegisteredPermission>, IdentityError>> + Send;

    /// Makes the registry exactly `declared`, atomically: new permissions are
    /// added, the others updated, and those no module declares any more are
    /// removed and withdrawn from every role.
    fn register(&self, declared: &[RegisteredPermission]) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
