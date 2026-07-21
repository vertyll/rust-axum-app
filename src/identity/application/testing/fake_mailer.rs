use std::sync::{Arc, Mutex};

use crate::identity::application::ports::IdentityMailer;
use crate::identity::domain::{Email, IdentityError, Username};

/// A recorded outgoing e-mail.
#[derive(Debug, Clone)]
pub(crate) struct SentEmail {
	pub kind: &'static str,
	pub to: String,
	pub token: String,
}

#[derive(Clone, Default)]
pub(crate) struct FakeMailer {
	sent: Arc<Mutex<Vec<SentEmail>>>,
	fail: Arc<Mutex<bool>>,
}

impl FakeMailer {
	pub fn sent(&self) -> Vec<SentEmail> {
		self.sent.lock().unwrap().clone()
	}

	pub fn last_token(&self) -> Option<String> {
		self.sent.lock().unwrap().last().map(|mail| mail.token.clone())
	}

	pub fn set_fail(&self, fail: bool) {
		*self.fail.lock().unwrap() = fail;
	}

	fn record(&self, kind: &'static str, to: &Email, token: &str) -> Result<(), IdentityError> {
		if *self.fail.lock().unwrap() {
			return Err(IdentityError::MailerFailure("test mailer down".into()));
		}
		let mail = SentEmail {
			kind,
			to: to.to_string(),
			token: token.to_string(),
		};
		self.sent.lock().unwrap().push(mail);
		Ok(())
	}
}

impl IdentityMailer for FakeMailer {
	async fn send_email_confirmation(
		&self,
		to: &Email,
		_username: &Username,
		token: &str,
	) -> Result<(), IdentityError> {
		self.record("confirmation", to, token)
	}

	async fn send_password_reset(&self, to: &Email, _username: &Username, token: &str) -> Result<(), IdentityError> {
		self.record("password_reset", to, token)
	}

	async fn send_email_change_confirmation(
		&self,
		to: &Email,
		_username: &Username,
		token: &str,
	) -> Result<(), IdentityError> {
		self.record("email_change", to, token)
	}
}
