//! Toasty adapter for the `SessionRepository` port.

use jiff::Timestamp;

use super::records::RefreshTokenRecord;
use super::user_repository::persistence;
use crate::identity::domain::{IdentityError, RefreshSession, SessionRepository, UserId};

#[derive(Clone)]
pub struct ToastySessionRepository {
	db: toasty::Db,
}

impl ToastySessionRepository {
	pub fn new(db: toasty::Db) -> Self {
		Self { db }
	}
}

impl SessionRepository for ToastySessionRepository {
	async fn create(&self, session: &RefreshSession) -> Result<(), IdentityError> {
		let mut db = self.db.clone();

		toasty::create!(RefreshTokenRecord {
			token: session.token.clone(),
			user_id: session.user_id.0,
			expires_at: session.expires_at,
		})
		.exec(&mut db)
		.await
		.map_err(persistence)?;

		Ok(())
	}

	async fn find_by_token(&self, token: &str) -> Result<Option<RefreshSession>, IdentityError> {
		let mut db = self.db.clone();

		let record = RefreshTokenRecord::filter_by_token(token)
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(record.map(|record| RefreshSession {
			token: record.token,
			user_id: UserId(record.user_id),
			expires_at: record.expires_at,
		}))
	}

	async fn delete(&self, user_id: UserId, token: &str) -> Result<(), IdentityError> {
		let mut db = self.db.clone();

		RefreshTokenRecord::filter_by_token(token)
			.filter(RefreshTokenRecord::fields().user_id().eq(user_id.0))
			.delete()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(())
	}

	async fn delete_all_for_user(&self, user_id: UserId) -> Result<(), IdentityError> {
		let mut db = self.db.clone();

		RefreshTokenRecord::filter_by_user_id(user_id.0)
			.delete()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(())
	}

	async fn delete_expired(&self, now: Timestamp) -> Result<(), IdentityError> {
		let mut db = self.db.clone();

		RefreshTokenRecord::filter(RefreshTokenRecord::fields().expires_at().lt(now))
			.delete()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(())
	}
}
