use crate::identity::domain::{Email, IdentityError, Username};

/// Outgoing identity e-mails. The port speaks in domain terms; template
/// rendering and SMTP are adapter details.
pub trait IdentityMailer: Clone + Send + Sync + 'static {
	fn send_email_confirmation(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> impl Future<Output = Result<(), IdentityError>> + Send;

	fn send_password_reset(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> impl Future<Output = Result<(), IdentityError>> + Send;

	fn send_email_change_confirmation(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
