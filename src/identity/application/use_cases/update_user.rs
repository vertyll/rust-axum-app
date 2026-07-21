use crate::identity::application::commands::UpdateUserProfile;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, User, UserId, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn update_user(&self, id: UserId, cmd: UpdateUserProfile) -> Result<User, IdentityError> {
		let mut user = self.get_user(id).await?;

		if let Some(username) = cmd.username {
			if let Some(existing) = self.users.find_by_username(&username).await? {
				if existing.id != id {
					return Err(IdentityError::UsernameTaken);
				}
			}
			user.username = username;
		}

		if let Some(email) = cmd.email {
			if let Some(existing) = self.users.find_by_email(&email).await? {
				if existing.id != id {
					return Err(IdentityError::EmailTaken);
				}
			}
			user.email = email;
		}

		self.users.update(&user).await?;
		Ok(user)
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::UpdateUserProfile;
	use crate::identity::application::testing::{harness, register_cmd};
	use crate::identity::domain::{IdentityError, Username};

	#[tokio::test]
	async fn renames_and_persists() {
		let h = harness();
		let user = h
			.service
			.create_user(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();

		let cmd = UpdateUserProfile {
			username: Some(Username::parse("renamed").unwrap()),
			email: None,
		};
		h.service.update_user(user.id, cmd).await.unwrap();
		assert_eq!(h.users.get(user.id).unwrap().username.as_str(), "renamed");
	}

	#[tokio::test]
	async fn rejects_username_taken_by_someone_else_but_allows_own() {
		let h = harness();
		let alice = h
			.service
			.create_user(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		h.service
			.create_user(register_cmd("bob", "b@example.com"))
			.await
			.unwrap();

		let taken = UpdateUserProfile {
			username: Some(Username::parse("bob").unwrap()),
			email: None,
		};
		let err = h.service.update_user(alice.id, taken).await.unwrap_err();
		assert!(matches!(err, IdentityError::UsernameTaken));

		let own = UpdateUserProfile {
			username: Some(Username::parse("alice").unwrap()),
			email: None,
		};
		assert!(h.service.update_user(alice.id, own).await.is_ok());
	}
}
