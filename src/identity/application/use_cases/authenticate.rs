use crate::identity::application::ports::{IdentityPorts, TokenService};
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::AccessClaims;
use crate::identity::domain::{IdentityError, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	/// Verifies a bearer token and re-checks the account state; used by the
	/// HTTP auth guard on every protected request.
	pub async fn authenticate(&self, bearer: &str) -> Result<AccessClaims, IdentityError> {
		let claims = self.tokens.verify_access(bearer)?;
		let user = self
			.users
			.find_by_id(claims.user_id())
			.await?
			.ok_or(IdentityError::UserNotFound)?;
		user.ensure_can_authenticate()?;
		Ok(claims)
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::commands::Credentials;
	use crate::identity::application::testing::{confirmed_user, harness};
	use crate::identity::domain::IdentityError;

	#[tokio::test]
	async fn returns_claims_for_a_live_account() {
		let h = harness();
		let user = confirmed_user(&h, "alice", "a@example.com").await;
		let credentials =
			Credentials { username: "alice".into(), password: "password123".into() };
		let (_, tokens) = h.service.login(credentials).await.unwrap();

		let claims = h.service.authenticate(&tokens.access_token).await.unwrap();
		assert_eq!(claims.user_id(), user.id);
	}

	#[tokio::test]
	async fn rejects_garbage_and_deactivated_accounts() {
		let h = harness();
		let garbage = h.service.authenticate("not-a-token").await.unwrap_err();
		assert!(matches!(garbage, IdentityError::InvalidToken));

		let mut user = confirmed_user(&h, "alice", "a@example.com").await;
		let credentials =
			Credentials { username: "alice".into(), password: "password123".into() };
		let (_, tokens) = h.service.login(credentials).await.unwrap();
		user.is_active = false;
		h.users.set(user);

		let inactive = h.service.authenticate(&tokens.access_token).await.unwrap_err();
		assert!(matches!(inactive, IdentityError::AccountInactive));
	}
}
