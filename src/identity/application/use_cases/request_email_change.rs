use crate::identity::application::commands::RequestEmailChange;
use crate::identity::application::ports::{IdentityMailer, IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::TokenKind;
use crate::identity::domain::{IdentityError, UserId, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn request_email_change(&self, user_id: UserId, cmd: RequestEmailChange) -> Result<(), IdentityError> {
		let mut user = self
			.users
			.find_by_id(user_id)
			.await?
			.ok_or(IdentityError::UserNotFound)?;

		if let Some(existing) = self.users.find_by_email(&cmd.new_email).await?
			&& existing.id != user_id
		{
			return Err(IdentityError::EmailTaken);
		}

		let token =
			self.tokens
				.sign_confirmation(TokenKind::EmailChange, user.id, &user.email, Some(&cmd.new_email))?;
		user.start_email_change(cmd.new_email, self.confirmation_token(&token))?;
		self.users.update(&user).await?;

		// Sent to the current address; retry-safe, so failures propagate.
		self.mailer
			.send_email_change_confirmation(&user.email, &user.username, &token)
			.await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::RequestEmailChange;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::{Email, IdentityError};

	fn cmd(email: &str) -> RequestEmailChange {
		RequestEmailChange {
			new_email: Email::parse(email).unwrap(),
		}
	}

	#[tokio::test]
	async fn stores_pending_change_and_mails_current_address() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		h.service
			.request_email_change(user.id, cmd("new@example.com"))
			.await
			.unwrap();
		let pending = h.users.get(user.id).unwrap().email_change.unwrap();
		assert_eq!(pending.new_email.as_str(), "new@example.com");

		let mail = h.mailer.sent().last().unwrap().clone();
		assert_eq!(mail.kind, "email_change");
		assert_eq!(mail.to, "a@example.com");
	}

	#[tokio::test]
	async fn rejects_taken_and_identical_addresses() {
		let h = harness();
		let alice = confirmed_user(&h, "alice", "a@example.com").await;
		confirmed_user(&h, "bob", "b@example.com").await;

		let taken = h.service.request_email_change(alice.id, cmd("b@example.com")).await;
		assert!(matches!(taken.unwrap_err(), IdentityError::EmailTaken));

		let same = h.service.request_email_change(alice.id, cmd("a@example.com")).await;
		assert!(matches!(same.unwrap_err(), IdentityError::SameEmailAsCurrent));
	}
}
