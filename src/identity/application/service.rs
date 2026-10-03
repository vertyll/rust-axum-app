//! The identity application service: sign-in through Keycloak, session
//! refresh and the mirror of Keycloak accounts the rest of the app joins to.

use crate::identity::application::ports::{IdentityPorts, KeycloakClient, TokenVerifier};
use crate::identity::application::session::{AuthSession, Caller, TokenPair};
use crate::identity::domain::{IdentityError, KeycloakIdentity, NewUser, User, UserId, UserRepository};

#[derive(Clone)]
pub struct IdentityService<P: IdentityPorts> {
	users: P::Users,
	keycloak: P::Keycloak,
	verifier: P::Verifier,
}

impl<P: IdentityPorts> IdentityService<P> {
	pub fn new(users: P::Users, keycloak: P::Keycloak, verifier: P::Verifier) -> Self {
		Self {
			users,
			keycloak,
			verifier,
		}
	}

	/// Redeems the code, then mirrors the account; a session Keycloak opened
	/// for an account that could not be stored is revoked again.
	pub async fn sign_in(&self, code: &str, code_verifier: &str) -> Result<AuthSession, IdentityError> {
		let tokens = self.keycloak.exchange(code, code_verifier).await?;
		let session = self.open(tokens, IdentityError::SignInRejected).await?;
		if let Err(err) = self.sync(&session.identity).await {
			self.keycloak.revoke(&session.refresh_token).await;
			return Err(err);
		}
		tracing::info!(keycloak_id = %session.identity.keycloak_id, "signed in");
		Ok(session)
	}

	pub async fn refresh(&self, session: &AuthSession) -> Result<AuthSession, IdentityError> {
		let tokens = self.keycloak.refresh(&session.refresh_token).await?;
		self.open(tokens, IdentityError::SessionExpired).await
	}

	pub async fn sign_out(&self, session: &AuthSession) {
		self.keycloak.revoke(&session.refresh_token).await;
		tracing::info!(keycloak_id = %session.identity.keycloak_id, "signed out");
	}

	/// Verifies an access token and resolves the local account behind it.
	pub async fn authenticate(&self, access_token: &str) -> Result<Caller, IdentityError> {
		let verified = self.verifier.verify(access_token).await?;
		let user = self.sync(&verified.identity).await?;
		Ok(Caller::from(&user))
	}

	pub async fn get_user(&self, id: UserId) -> Result<User, IdentityError> {
		self.users.find_by_id(id).await?.ok_or(IdentityError::UserNotFound)
	}

	pub async fn list_users(&self) -> Result<Vec<User>, IdentityError> {
		self.users.list().await
	}

	async fn sync(&self, identity: &KeycloakIdentity) -> Result<User, IdentityError> {
		match self.users.find_by_keycloak_id(&identity.keycloak_id).await? {
			Some(mut user) => {
				if user.mirror(identity) {
					self.users.update(&user).await?;
				}
				Ok(user)
			}
			None => self.users.create(NewUser::from(identity)).await,
		}
	}

	async fn open(&self, tokens: TokenPair, on_rejection: IdentityError) -> Result<AuthSession, IdentityError> {
		let verified = self
			.verifier
			.verify(&tokens.access_token)
			.await
			.map_err(|err| match err {
				IdentityError::InvalidToken => on_rejection,
				other => other,
			})?;
		Ok(AuthSession {
			identity: verified.identity,
			access_token: tokens.access_token,
			refresh_token: tokens.refresh_token,
			access_token_expires_at: verified.expires_at,
		})
	}
}

#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{Harness, identity};
	use crate::identity::domain::{IdentityError, RoleName, UserRepository};

	#[tokio::test]
	async fn signing_in_creates_the_account_once() {
		let harness = Harness::new();
		let first = harness.service.sign_in("code", "verifier").await.unwrap();
		harness.service.sign_in("code", "verifier").await.unwrap();

		assert_eq!(first.refresh_token, "refresh-1");
		assert_eq!(harness.users.list().await.unwrap().len(), 1);
	}

	#[tokio::test]
	async fn a_failed_account_write_revokes_the_new_session() {
		let harness = Harness::new();
		harness.users.fail_writes();

		let result = harness.service.sign_in("code", "verifier").await;

		assert!(matches!(result, Err(IdentityError::PersistenceFailure(_))));
		assert_eq!(harness.keycloak.revoked(), vec!["refresh-1".to_string()]);
	}

	#[tokio::test]
	async fn a_token_keycloak_rejects_ends_the_sign_in() {
		let harness = Harness::new();
		harness.verifier.reject_all();

		let result = harness.service.sign_in("code", "verifier").await;

		assert!(matches!(result, Err(IdentityError::SignInRejected)));
	}

	#[tokio::test]
	async fn authenticating_mirrors_role_changes() {
		let harness = Harness::new();
		harness.service.sign_in("code", "verifier").await.unwrap();
		harness.verifier.issue(identity(vec![RoleName::Admin]));

		let caller = harness.service.authenticate("access").await.unwrap();

		assert!(caller.has_role(RoleName::Admin));
		assert!(harness.users.list().await.unwrap()[0].has_role(RoleName::Admin));
	}

	#[tokio::test]
	async fn an_unknown_user_is_not_found() {
		let harness = Harness::new();
		let result = harness.service.get_user(crate::identity::domain::UserId(42)).await;
		assert!(matches!(result, Err(IdentityError::UserNotFound)));
	}
}
