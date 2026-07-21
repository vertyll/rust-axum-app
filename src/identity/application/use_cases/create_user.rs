use crate::identity::application::commands::RegisterUser;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, User};

impl<P: IdentityPorts> IdentityService<P> {
	/// Admin-driven account creation: same flow as `register`, no session.
	pub async fn create_user(&self, cmd: RegisterUser) -> Result<User, IdentityError> {
		self.create_account(cmd).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{harness, register_cmd};
	use crate::identity::domain::IdentityError;

	#[tokio::test]
	async fn creates_account_without_opening_a_session() {
		let h = harness();
		let user = h
			.service
			.create_user(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		assert!(h.users.get(user.id).is_some());
		assert_eq!(h.sessions.count(), 0);
	}

	#[tokio::test]
	async fn rejects_taken_username() {
		let h = harness();
		h.service
			.create_user(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		let err = h
			.service
			.create_user(register_cmd("alice", "b@example.com"))
			.await
			.unwrap_err();
		assert!(matches!(err, IdentityError::UsernameTaken));
	}
}
