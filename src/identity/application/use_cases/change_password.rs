use crate::identity::application::commands::ChangePassword;
use crate::identity::application::ports::{IdentityPorts, PasswordHasher};
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, UserId, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn change_password(&self, user_id: UserId, cmd: ChangePassword) -> Result<(), IdentityError> {
		let mut user = self
			.users
			.find_by_id(user_id)
			.await?
			.ok_or(IdentityError::UserNotFound)?;

		let current_ok = self
			.hasher
			.verify(cmd.current_password, user.password_hash.clone())
			.await?;
		if !current_ok {
			return Err(IdentityError::InvalidCurrentPassword);
		}

		let new_hash = self.hasher.hash(cmd.new_password).await?;
		user.set_password(new_hash);
		self.users.update(&user).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::ChangePassword;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::IdentityError;

	#[tokio::test]
	async fn replaces_hash_after_verifying_current() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		let cmd = ChangePassword {
			current_password: "password123".into(),
			new_password: "even-better-pass".into(),
		};
		h.service.change_password(user.id, cmd).await.unwrap();
		let stored = h.users.get(user.id).unwrap();
		assert_eq!(stored.password_hash.as_str(), "hashed:even-better-pass");
	}

	#[tokio::test]
	async fn rejects_wrong_current_password() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		let cmd = ChangePassword {
			current_password: "guess".into(),
			new_password: "whatever-else".into(),
		};
		let err = h.service.change_password(user.id, cmd).await.unwrap_err();
		assert!(matches!(err, IdentityError::InvalidCurrentPassword));
	}
}
