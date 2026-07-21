use jiff::Timestamp;

use crate::identity::application::commands::ResetPassword;
use crate::identity::application::ports::{IdentityPorts, PasswordHasher, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::TokenKind;
use crate::identity::domain::{IdentityError, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn reset_password(&self, cmd: ResetPassword) -> Result<(), IdentityError> {
		let claims = self.tokens.verify_confirmation(&cmd.token)?;
		if claims.kind != TokenKind::PasswordReset {
			return Err(IdentityError::InvalidTokenType);
		}

		let mut user = self
			.users
			.find_by_id(claims.user_id())
			.await?
			.ok_or(IdentityError::UserNotFound)?;

		let new_hash = self.hasher.hash(cmd.new_password).await?;
		user.complete_password_reset(&cmd.token, new_hash, Timestamp::now())?;
		self.users.update(&user).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::{RequestPasswordReset, ResetPassword};
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::{Email, IdentityError};

	#[tokio::test]
	async fn resets_with_the_mailed_token() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let request =
			RequestPasswordReset { email: Email::parse("a@example.com").unwrap() };
		h.service.request_password_reset(request).await.unwrap();
		let token = h.mailer.last_token().unwrap();

		let cmd = ResetPassword { token, new_password: "brand-new-pass".into() };
		h.service.reset_password(cmd).await.unwrap();

		let stored = h.users.get(user.id).unwrap();
		assert_eq!(stored.password_hash.as_str(), "hashed:brand-new-pass");
		assert!(stored.password_reset.is_none());
	}

	#[tokio::test]
	async fn rejects_tokens_of_another_kind() {
		let h = harness();
		confirmed_user(&h, "alice", "a@example.com").await;
		// The registration e-mail carried an *email-confirmation* token.
		let confirmation = h.mailer.last_token().unwrap();

		let cmd = ResetPassword { token: confirmation, new_password: "whatever-pass".into() };
		let err = h.service.reset_password(cmd).await.unwrap_err();
		assert!(matches!(err, IdentityError::InvalidTokenType));
	}
}
