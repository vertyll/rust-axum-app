//! `UserRepository` — the User aggregate's driven port, implemented by the
//! Toasty adapter and wired through generics: native `async fn` in traits,
//! static dispatch, no `async_trait` boxing.

use super::error::IdentityError;
use super::user::{KeycloakId, NewUser, User, UserId};

pub trait UserRepository: Clone + Send + Sync + 'static {
	/// Persists a new account atomically (user row + role links).
	fn create(&self, user: NewUser) -> impl Future<Output = Result<User, IdentityError>> + Send;

	fn find_by_id(&self, id: UserId) -> impl Future<Output = Result<Option<User>, IdentityError>> + Send;

	fn find_by_keycloak_id(
		&self,
		keycloak_id: &KeycloakId,
	) -> impl Future<Output = Result<Option<User>, IdentityError>> + Send;

	fn list(&self) -> impl Future<Output = Result<Vec<User>, IdentityError>> + Send;

	/// Writes the aggregate back atomically, role links included.
	fn update(&self, user: &User) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
