use crate::identity::application::commands::Credentials;
use crate::identity::application::ports::{IdentityPorts, PasswordHasher};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::AuthTokens;
use crate::identity::domain::{IdentityError, User, UserRepository, Username};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn login(&self, cmd: Credentials) -> Result<(User, AuthTokens), IdentityError> {
		// All lookup failures collapse into `InvalidCredentials` so responses
		// do not reveal whether a username exists.
		let username =
			Username::parse(cmd.username).map_err(|_| IdentityError::InvalidCredentials)?;
		let user = self
			.users
			.find_by_username(&username)
			.await?
			.ok_or(IdentityError::InvalidCredentials)?;

		user.ensure_can_authenticate()?;

		let valid = self.hasher.verify(cmd.password, user.password_hash.clone()).await?;
		if !valid {
			return Err(IdentityError::InvalidCredentials);
		}

		let tokens = self.open_session(&user).await?;
		Ok((user, tokens))
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness, register_cmd};
	use crate::identity::domain::IdentityError;

	fn creds(username: &str, password: &str) -> Credentials {
		Credentials { username: username.into(), password: password.into() }
	}

	#[tokio::test]
	async fn succeeds_for_confirmed_account_and_opens_session() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		let (logged_in, tokens) =
			h.service.login(creds("alice", "password123")).await.unwrap();
		assert_eq!(logged_in.id, user.id);
		assert_eq!(h.sessions.count(), 1);
		assert!(tokens.access_token.starts_with("access|"));
	}

	#[tokio::test]
	async fn wrong_password_and_unknown_user_look_identical() {
		let h = harness();
		confirmed_user(&h, "alice", "a@example.com").await;

		let wrong = h.service.login(creds("alice", "nope-nope")).await.unwrap_err();
		assert!(matches!(wrong, IdentityError::InvalidCredentials));

		let unknown = h.service.login(creds("ghost", "password123")).await.unwrap_err();
		assert!(matches!(unknown, IdentityError::InvalidCredentials));
	}

	#[tokio::test]
	async fn rejects_unconfirmed_and_inactive_accounts() {
		let h = harness();
		h.service.create_user(register_cmd("fresh", "f@example.com")).await.unwrap();
		let unconfirmed = h.service.login(creds("fresh", "password123")).await.unwrap_err();
		assert!(matches!(unconfirmed, IdentityError::EmailNotConfirmed));

		let mut user = confirmed_user(&h, "gone", "g@example.com").await;
		user.is_active = false;
		h.users.set(user);
		let inactive = h.service.login(creds("gone", "password123")).await.unwrap_err();
		assert!(matches!(inactive, IdentityError::AccountInactive));
	}
}
