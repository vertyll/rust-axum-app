use super::{KeycloakClient, TokenVerifier};
use crate::identity::domain::UserRepository;

/// A type-level bundle naming which adapter satisfies each port.
pub trait IdentityPorts: Clone + Send + Sync + 'static {
	type Users: UserRepository;
	type Keycloak: KeycloakClient;
	type Verifier: TokenVerifier;
}
