use crate::identity::application::token::{AccessClaims, ConfirmationClaims, TokenKind};
use crate::identity::domain::{Email, IdentityError, User, UserId};

/// Signing and verification of the module's tokens (access + confirmation).
/// Signing is synchronous — it is cheap, pure CPU work with no IO.
pub trait TokenService: Clone + Send + Sync + 'static {
	fn sign_access(&self, user: &User) -> Result<String, IdentityError>;
	fn verify_access(&self, token: &str) -> Result<AccessClaims, IdentityError>;

	fn confirmation_ttl_seconds(&self) -> i64;
	fn sign_confirmation(
		&self,
		kind: TokenKind,
		user_id: UserId,
		email: &Email,
		new_email: Option<&Email>,
	) -> Result<String, IdentityError>;
	fn verify_confirmation(&self, token: &str) -> Result<ConfirmationClaims, IdentityError>;
}
