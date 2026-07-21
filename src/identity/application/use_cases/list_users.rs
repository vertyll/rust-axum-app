use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, User, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn list_users(&self) -> Result<Vec<User>, IdentityError> {
		self.users.list().await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{harness, register_cmd};

	#[tokio::test]
	async fn returns_every_account() {
		let h = harness();
		h.service
			.create_user(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		h.service
			.create_user(register_cmd("bob", "b@example.com"))
			.await
			.unwrap();
		assert_eq!(h.service.list_users().await.unwrap().len(), 2);
	}
}
