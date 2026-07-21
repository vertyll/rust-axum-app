use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, SessionRepository, UserId};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn logout(&self, user_id: UserId, refresh_token: &str) -> Result<(), IdentityError> {
		self.sessions.delete(user_id, refresh_token).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness};

	#[tokio::test]
	async fn removes_only_the_presented_session() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let creds = || Credentials {
			username: "alice".into(),
			password: "password123".into(),
		};
		let (_, first) = h.service.login(creds()).await.unwrap();
		h.service.login(creds()).await.unwrap();
		assert_eq!(h.sessions.count(), 2);

		h.service.logout(user.id, &first.refresh_token).await.unwrap();
		assert_eq!(h.sessions.count(), 1);
	}
}
