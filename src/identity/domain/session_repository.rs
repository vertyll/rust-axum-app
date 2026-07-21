//! `SessionRepository` — the RefreshSession aggregate's driven port.

use jiff::Timestamp;

use super::error::IdentityError;
use super::session::RefreshSession;
use super::user::UserId;

pub trait SessionRepository: Clone + Send + Sync + 'static {
	fn create(&self, session: &RefreshSession) -> impl Future<Output = Result<(), IdentityError>> + Send;

	fn find_by_token(&self, token: &str) -> impl Future<Output = Result<Option<RefreshSession>, IdentityError>> + Send;

	fn delete(&self, user_id: UserId, token: &str) -> impl Future<Output = Result<(), IdentityError>> + Send;

	fn delete_all_for_user(&self, user_id: UserId) -> impl Future<Output = Result<(), IdentityError>> + Send;

	fn delete_expired(&self, now: Timestamp) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
