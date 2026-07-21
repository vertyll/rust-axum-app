use std::sync::{Arc, Mutex};

use jiff::Timestamp;

use crate::identity::domain::{IdentityError, RefreshSession, SessionRepository, UserId};

#[derive(Clone, Default)]
pub(crate) struct InMemorySessions {
	sessions: Arc<Mutex<Vec<RefreshSession>>>,
}

impl InMemorySessions {
	pub fn count(&self) -> usize {
		self.sessions.lock().unwrap().len()
	}

	pub fn insert(&self, session: RefreshSession) {
		self.sessions.lock().unwrap().push(session);
	}
}

impl SessionRepository for InMemorySessions {
	async fn create(&self, session: &RefreshSession) -> Result<(), IdentityError> {
		self.sessions.lock().unwrap().push(session.clone());
		Ok(())
	}

	async fn find_by_token(&self, token: &str) -> Result<Option<RefreshSession>, IdentityError> {
		Ok(self.sessions.lock().unwrap().iter().find(|s| s.token == token).cloned())
	}

	async fn delete(&self, user_id: UserId, token: &str) -> Result<(), IdentityError> {
		self.sessions
			.lock()
			.unwrap()
			.retain(|s| !(s.user_id == user_id && s.token == token));
		Ok(())
	}

	async fn delete_all_for_user(&self, user_id: UserId) -> Result<(), IdentityError> {
		self.sessions.lock().unwrap().retain(|s| s.user_id != user_id);
		Ok(())
	}

	async fn delete_expired(&self, now: Timestamp) -> Result<(), IdentityError> {
		self.sessions.lock().unwrap().retain(|s| !s.is_expired(now));
		Ok(())
	}
}
