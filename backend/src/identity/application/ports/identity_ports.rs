use super::{RealmAdmin, TokenEndpoint, TokenVerifier};
use crate::identity::domain::{PermissionRegistry, RoleRepository, UserRepository};

/// A type-level bundle naming which adapter satisfies each port.
pub trait IdentityPorts: Clone + Send + Sync + 'static {
    type Users: UserRepository;
    type Roles: RoleRepository;
    type Permissions: PermissionRegistry;
    type Tokens: TokenEndpoint;
    type Realm: RealmAdmin;
    type Verifier: TokenVerifier;
}
