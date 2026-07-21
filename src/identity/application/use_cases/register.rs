use crate::identity::application::commands::RegisterUser;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::AuthTokens;
use crate::identity::domain::{IdentityError, User};

impl<P: IdentityPorts> IdentityService<P> {
	/// Self-service registration: creates the account and opens a session.
	pub async fn register(&self, cmd: RegisterUser) -> Result<(User, AuthTokens), IdentityError> {
		let user = self.create_account(cmd).await?;
		let tokens = self.open_session(&user).await?;
		Ok((user, tokens))
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{harness, register_cmd};
	use crate::identity::domain::IdentityError;

	#[tokio::test]
	async fn creates_account_opens_session_and_mails_token() {
		let h = harness();
		let (user, tokens) = h
			.service
			.register(register_cmd("alice", "alice@example.com"))
			.await
			.unwrap();

		assert!(!user.is_email_confirmed);
		assert!(h.users.get(user.id).unwrap().email_confirmation.is_some());
		assert_eq!(h.sessions.count(), 1);
		assert!(tokens.access_token.starts_with("access|"));

		let sent = h.mailer.sent();
		assert_eq!(sent.len(), 1);
		assert_eq!(sent[0].kind, "confirmation");
	}

	#[tokio::test]
	async fn rejects_taken_email() {
		let h = harness();
		h.service
			.register(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		let err = h
			.service
			.register(register_cmd("bob", "a@example.com"))
			.await
			.unwrap_err();
		assert!(matches!(err, IdentityError::EmailTaken));
	}

	#[tokio::test]
	async fn mailer_failure_does_not_roll_back_registration() {
		let h = harness();
		h.mailer.set_fail(true);
		let (user, _) = h
			.service
			.register(register_cmd("alice", "a@example.com"))
			.await
			.unwrap();
		assert!(h.users.get(user.id).is_some());
		assert!(h.mailer.sent().is_empty());
	}
}
