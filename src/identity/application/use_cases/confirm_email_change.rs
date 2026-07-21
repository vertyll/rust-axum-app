use jiff::Timestamp;

use crate::identity::application::ports::{IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::TokenKind;
use crate::identity::domain::{IdentityError, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn confirm_email_change(&self, token: &str) -> Result<(), IdentityError> {
		let claims = self.tokens.verify_confirmation(token)?;
		if claims.kind != TokenKind::EmailChange {
			return Err(IdentityError::InvalidTokenType);
		}

		let mut user = self
			.users
			.find_by_id(claims.user_id())
			.await?
			.ok_or(IdentityError::UserNotFound)?;

		let previous_email = user.complete_email_change(token, Timestamp::now())?;
		self.users.save_email_change(&user, &previous_email).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::RequestEmailChange;
	use crate::identity::application::ports::TokenService;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::application::token::TokenKind;
	use crate::identity::domain::{Email, IdentityError};

	#[tokio::test]
	async fn swaps_the_address_and_records_history() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let request =
			RequestEmailChange { new_email: Email::parse("new@example.com").unwrap() };
		h.service.request_email_change(user.id, request).await.unwrap();
		let token = h.mailer.last_token().unwrap();

		h.service.confirm_email_change(&token).await.unwrap();

		assert_eq!(h.users.get(user.id).unwrap().email.as_str(), "new@example.com");
		let history = h.users.email_history();
		assert_eq!(history.len(), 1);
		assert_eq!(history[0].1.as_str(), "a@example.com");
	}

	#[tokio::test]
	async fn rejects_tokens_of_another_kind() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let reset = h
			.tokens
			.sign_confirmation(TokenKind::PasswordReset, user.id, &user.email, None)
			.unwrap();

		let err = h.service.confirm_email_change(&reset).await.unwrap_err();
		assert!(matches!(err, IdentityError::InvalidTokenType));
	}
}
