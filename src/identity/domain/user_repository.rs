//! `UserRepository` — the User aggregate's driven port, implemented by the
//! Toasty adapter and wired through generics: native `async fn` in traits,
//! static dispatch, no `async_trait` boxing. Transactional rule: one
//! aggregate = one transaction inside the adapter; multi-row atomic writes
//! are single port methods and no transaction handle ever escapes.

use super::error::IdentityError;
use super::user::{Email, NewUser, User, UserId, Username};

pub trait UserRepository: Clone + Send + Sync + 'static {
	/// Persists a new account atomically (user row + role links).
	fn create(&self, user: NewUser) -> impl Future<Output = Result<User, IdentityError>> + Send;

	fn find_by_id(
		&self,
		id: UserId,
	) -> impl Future<Output = Result<Option<User>, IdentityError>> + Send;

	fn find_by_email(
		&self,
		email: &Email,
	) -> impl Future<Output = Result<Option<User>, IdentityError>> + Send;

	fn find_by_username(
		&self,
		username: &Username,
	) -> impl Future<Output = Result<Option<User>, IdentityError>> + Send;

	fn list(&self) -> impl Future<Output = Result<Vec<User>, IdentityError>> + Send;

	/// Writes the aggregate's current state back (everything except roles).
	fn update(&self, user: &User) -> impl Future<Output = Result<(), IdentityError>> + Send;

	/// Persists a completed e-mail change atomically: the updated user row
	/// plus an audit entry recording `previous_email`.
	fn save_email_change(
		&self,
		user: &User,
		previous_email: &Email,
	) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
