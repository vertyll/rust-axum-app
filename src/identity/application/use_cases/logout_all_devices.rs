use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, SessionRepository, UserId};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn logout_all_devices(&self, user_id: UserId) -> Result<(), IdentityError> {
		self.sessions.delete_all_for_user(user_id).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness};

	#[tokio::test]
	async fn removes_every_session_of_one_user_only() {
		let h = harness();
		let alice = confirmed_user(&h, "alice", "a@example.com").await;
		confirmed_user(&h, "bob", "b@example.com").await;
		let login = |u: &str| Credentials {
			username: u.into(),
			password: "password123".into(),
		};
		h.service.login(login("alice")).await.unwrap();
		h.service.login(login("alice")).await.unwrap();
		h.service.login(login("bob")).await.unwrap();

		h.service.logout_all_devices(alice.id).await.unwrap();
		assert_eq!(h.sessions.count(), 1);
	}
}
