use crate::identity::application::commands::RequestPasswordReset;
use crate::identity::application::ports::{IdentityMailer, IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::TokenKind;
use crate::identity::domain::{IdentityError, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn request_password_reset(&self, cmd: RequestPasswordReset) -> Result<(), IdentityError> {
		let mut user = self
			.users
			.find_by_email(&cmd.email)
			.await?
			.ok_or(IdentityError::UserNotFound)?;

		let token = self
			.tokens
			.sign_confirmation(TokenKind::PasswordReset, user.id, &user.email, None)?;
		user.start_password_reset(self.confirmation_token(&token));
		self.users.update(&user).await?;

		// Mailer failure is retry-safe (a new request overwrites the token),
		// so it is propagated instead of swallowed.
		self.mailer
			.send_password_reset(&user.email, &user.username, &token)
			.await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::RequestPasswordReset;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::{Email, IdentityError};

	fn cmd(email: &str) -> RequestPasswordReset {
		RequestPasswordReset {
			email: Email::parse(email).unwrap(),
		}
	}

	#[tokio::test]
	async fn stores_token_and_mails_it() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		h.service.request_password_reset(cmd("a@example.com")).await.unwrap();
		assert!(h.users.get(user.id).unwrap().password_reset.is_some());
		assert_eq!(h.mailer.sent().last().unwrap().kind, "password_reset");
	}

	#[tokio::test]
	async fn unknown_email_and_mailer_failure_surface() {
		let h = harness();
		let missing = h
			.service
			.request_password_reset(cmd("x@example.com"))
			.await
			.unwrap_err();
		assert!(matches!(missing, IdentityError::UserNotFound));

		confirmed_user(&h, "alice", "a@example.com").await;
		h.mailer.set_fail(true);
		let down = h
			.service
			.request_password_reset(cmd("a@example.com"))
			.await
			.unwrap_err();
		assert!(matches!(down, IdentityError::MailerFailure(_)));
	}
}
