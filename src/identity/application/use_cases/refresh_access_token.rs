use jiff::Timestamp;

use crate::identity::application::ports::{IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, SessionRepository, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	/// Works without an access token — refreshing is needed exactly when the
	/// access token has already expired.
	pub async fn refresh_access_token(&self, refresh_token: &str) -> Result<String, IdentityError> {
		let session = self
			.sessions
			.find_by_token(refresh_token)
			.await?
			.ok_or(IdentityError::RefreshTokenInvalid)?;

		if session.is_expired(Timestamp::now()) {
			return Err(IdentityError::RefreshTokenExpired);
		}

		let user = self
			.users
			.find_by_id(session.user_id)
			.await?
			.ok_or(IdentityError::RefreshTokenInvalid)?;
		user.ensure_can_authenticate()?;

		self.tokens.sign_access(&user)
	}
}
#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::{IdentityError, RefreshSession};

	#[tokio::test]
	async fn issues_a_fresh_access_token() {
		let h = harness();
		confirmed_user(&h, "alice", "a@example.com").await;
		let credentials =
			Credentials { username: "alice".into(), password: "password123".into() };
		let (_, tokens) = h.service.login(credentials).await.unwrap();

		let access = h.service.refresh_access_token(&tokens.refresh_token).await.unwrap();
		assert!(access.starts_with("access|"));
	}

	#[tokio::test]
	async fn rejects_unknown_and_expired_tokens() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;

		let unknown = h.service.refresh_access_token("nope").await.unwrap_err();
		assert!(matches!(unknown, IdentityError::RefreshTokenInvalid));

		h.sessions.insert(RefreshSession {
			token: "stale".into(),
			user_id: user.id,
			expires_at: Timestamp::UNIX_EPOCH,
		});
		let expired = h.service.refresh_access_token("stale").await.unwrap_err();
		assert!(matches!(expired, IdentityError::RefreshTokenExpired));
	}
}
