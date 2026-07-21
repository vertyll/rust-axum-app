use jiff::Timestamp;

use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, SessionRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn clean_expired_sessions(&self) -> Result<(), IdentityError> {
		self.sessions.delete_expired(Timestamp::now()).await
	}
}
#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::RefreshSession;

	#[tokio::test]
	async fn drops_expired_keeps_live() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let credentials = Credentials {
			username: "alice".into(),
			password: "password123".into(),
		};
		h.service.login(credentials).await.unwrap();
		h.sessions.insert(RefreshSession {
			token: "stale".into(),
			user_id: user.id,
			expires_at: Timestamp::UNIX_EPOCH,
		});

		h.service.clean_expired_sessions().await.unwrap();
		assert_eq!(h.sessions.count(), 1);
	}
}
