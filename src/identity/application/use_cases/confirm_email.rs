use jiff::Timestamp;

use crate::identity::application::ports::{IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::TokenKind;
use crate::identity::domain::{IdentityError, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn confirm_email(&self, token: &str) -> Result<(), IdentityError> {
		let claims = self.tokens.verify_confirmation(token)?;
		if claims.kind != TokenKind::EmailConfirmation {
			return Err(IdentityError::InvalidTokenType);
		}

		let mut user = self
			.users
			.find_by_id(claims.user_id())
			.await?
			.ok_or(IdentityError::UserNotFound)?;
		user.confirm_email(token, Timestamp::now())?;
		self.users.update(&user).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::ports::TokenService;
	use crate::identity::application::testing::{harness, register_cmd};
	use crate::identity::application::token::TokenKind;
	use crate::identity::domain::IdentityError;

	#[tokio::test]
	async fn confirms_with_the_mailed_token() {
		let h = harness();
		let user = h.service.create_user(register_cmd("alice", "a@example.com")).await.unwrap();
		let token = h.mailer.last_token().unwrap();

		h.service.confirm_email(&token).await.unwrap();
		let stored = h.users.get(user.id).unwrap();
		assert!(stored.is_email_confirmed);
		assert!(stored.email_confirmation.is_none());
	}

	#[tokio::test]
	async fn rejects_wrong_kind_and_non_stored_tokens() {
		let h = harness();
		let user = h.service.create_user(register_cmd("alice", "a@example.com")).await.unwrap();

		let reset = h
			.tokens
			.sign_confirmation(TokenKind::PasswordReset, user.id, &user.email, None)
			.unwrap();
		let kind = h.service.confirm_email(&reset).await.unwrap_err();
		assert!(matches!(kind, IdentityError::InvalidTokenType));

		let fresh = h
			.tokens
			.sign_confirmation(TokenKind::EmailConfirmation, user.id, &user.email, None)
			.unwrap();
		let mismatch = h.service.confirm_email(&fresh).await.unwrap_err();
		assert!(matches!(mismatch, IdentityError::InvalidToken));
	}
}
