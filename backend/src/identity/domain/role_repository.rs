//! `RoleRepository` — the roles and what they grant.

use super::error::IdentityError;
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
}
