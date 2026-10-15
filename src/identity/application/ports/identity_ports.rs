use super::{KeycloakClient, RealmAdmin, TokenVerifier};
use crate::identity::domain::{RoleRepository, UserRepository};

/// A type-level bundle naming which adapter satisfies each port.
pub trait IdentityPorts: Clone + Send + Sync + 'static {
    type Users: UserRepository;
    type Roles: RoleRepository;
    type Keycloak: KeycloakClient;
    type Realm: RealmAdmin;
    type Verifier: TokenVerifier;
}
