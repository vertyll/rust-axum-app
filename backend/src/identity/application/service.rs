//! The identity application service: sign-in through Keycloak, session
//! refresh and the mirror of Keycloak accounts the rest of the app joins to.
//! Roles and what they grant are managed in [`roles`](super::roles).

use crate::identity::application::ports::{IdentityPorts, KeycloakClient, RealmAdmin, SecondFactor, TokenVerifier};
use crate::identity::application::session::{AuthSession, Caller, TokenPair};
use crate::identity::domain::{
    Grants, IdentityError, KeycloakId, KeycloakIdentity, NewUser, Role, RoleRepository, User, UserId, UserRepository,
};
use crate::shared_kernel::pagination::{PageRequest, PageResult};

#[derive(Clone)]
pub struct IdentityService<P: IdentityPorts> {
    pub(super) users: P::Users,
    pub(super) roles: P::Roles,
    keycloak: P::Keycloak,
    pub(super) realm: P::Realm,
    verifier: P::Verifier,
}

impl<P: IdentityPorts> IdentityService<P> {
    pub fn new(
        users: P::Users,
        roles: P::Roles,
        keycloak: P::Keycloak,
        realm: P::Realm,
        verifier: P::Verifier,
    ) -> Self {
        Self {
            users,
            roles,
            keycloak,
            realm,
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

    /// Verifies an access token, resolves the local account behind it and
    /// what its roles grant.
    pub async fn authenticate(&self, access_token: &str) -> Result<Caller, IdentityError> {
        let verified = self.verifier.verify(access_token).await?;
        let (user, roles) = self.sync(&verified.identity).await?;
        Ok(Caller::new(&user, Grants::of(&roles)))
    }

    /// The caller's second factor, read from Keycloak with their own access token.
    pub async fn second_factor(&self, access_token: &str) -> Result<Option<SecondFactor>, IdentityError> {
        self.keycloak.second_factor(access_token).await
    }

    pub async fn get_user(&self, id: UserId) -> Result<User, IdentityError> {
        self.users.find_by_id(id).await?.ok_or(IdentityError::UserNotFound)
    }

    pub async fn list_users(&self, request: PageRequest) -> Result<PageResult<User>, IdentityError> {
        self.users.page(request).await
    }

    /// Makes Keycloak show the signed-in person's account pages, such as setting
    /// up a second factor, in the language they use here. Keycloak prefers the
    /// language stored on the account to the one a sign-in asks for, so the
    /// account's is set first; when that fails, the pages open in the stored one.
    pub async fn keep_account_language(&self, account: &KeycloakId, language: &str) {
        if let Err(err) = self.realm.set_language(account, language).await {
            tracing::warn!(keycloak_id = %account, "could not store the account's language in Keycloak: {err}");
        }
    }

    /// Mirrors the account and the roles of the token this application
    /// knows; a realm role with no role here, such as Keycloak's own
    /// `offline_access`, grants nothing and is left out.
    async fn sync(&self, identity: &KeycloakIdentity) -> Result<(User, Vec<Role>), IdentityError> {
        let roles = self.roles.find_many(&identity.roles).await?;
        let mut known = identity.clone();
        known.roles = roles.iter().map(|role| role.name.clone()).collect();
        known.roles.sort();
        let user = match self.users.find_by_keycloak_id(&known.keycloak_id).await? {
            Some(mut user) => {
                if user.mirror(&known) {
                    self.users.update(&user).await?;
                }
                user
            }
            None => self.users.create(NewUser::from(&known)).await?,
        };
        Ok((user, roles))
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

    fn role(name: &str) -> RoleName {
        RoleName::parse(name).unwrap()
    }

    #[tokio::test]
    async fn the_second_factor_is_the_one_keycloak_holds() {
        let harness = Harness::new().await;
        assert_eq!(harness.service.second_factor("access").await.unwrap(), None);

        harness.keycloak.set_second_factor("otp-1");

        let factor = harness.service.second_factor("access").await.unwrap().unwrap();
        assert_eq!(factor.credential_id, "otp-1");
    }

    #[tokio::test]
    async fn the_account_pages_follow_the_language_used_here() {
        let harness = Harness::new().await;
        let session = harness.service.sign_in("code", "verifier").await.unwrap();

        harness
            .service
            .keep_account_language(&session.identity.keycloak_id, "pl")
            .await;

        assert_eq!(
            harness.realm.calls().last(),
            Some(&format!("language {} pl", session.identity.keycloak_id))
        );
    }

    #[tokio::test]
    async fn a_language_keycloak_does_not_store_leaves_the_account_pages_as_they_are() {
        let harness = Harness::new().await;
        let session = harness.service.sign_in("code", "verifier").await.unwrap();
        let before = harness.realm.calls();
        harness.realm.fail();

        harness
            .service
            .keep_account_language(&session.identity.keycloak_id, "pl")
            .await;

        assert_eq!(harness.realm.calls(), before);
    }

    #[tokio::test]
    async fn signing_in_creates_the_account_once() {
        let harness = Harness::new().await;
        let first = harness.service.sign_in("code", "verifier").await.unwrap();
        harness.service.sign_in("code", "verifier").await.unwrap();

        assert_eq!(first.refresh_token, "refresh-1");
        assert_eq!(harness.users.list().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn a_failed_account_write_revokes_the_new_session() {
        let harness = Harness::new().await;
        harness.users.fail_writes();

        let result = harness.service.sign_in("code", "verifier").await;

        assert!(matches!(result, Err(IdentityError::PersistenceFailure(_))));
        assert_eq!(harness.keycloak.revoked(), vec!["refresh-1".to_string()]);
    }

    #[tokio::test]
    async fn a_token_keycloak_rejects_ends_the_sign_in() {
        let harness = Harness::new().await;
        harness.verifier.reject_all();

        let result = harness.service.sign_in("code", "verifier").await;

        assert!(matches!(result, Err(IdentityError::SignInRejected)));
    }

    #[tokio::test]
    async fn authenticating_mirrors_role_changes_and_grants_what_they_grant() {
        let harness = Harness::new().await;
        harness.service.sign_in("code", "verifier").await.unwrap();
        harness.verifier.issue(identity(vec![role("admin")]));

        let caller = harness.service.authenticate("access").await.unwrap();

        assert!(caller.has_role(&role("admin")));
        assert!(caller.grants.allows("ANYTHING"));
        assert!(harness.users.list().await.unwrap()[0].has_role(&role("admin")));
    }

    #[tokio::test]
    async fn a_realm_role_unknown_here_is_left_out() {
        let harness = Harness::new().await;
        harness
            .verifier
            .issue(identity(vec![role("offline_access"), role("user")]));

        let caller = harness.service.authenticate("access").await.unwrap();

        assert_eq!(caller.roles, vec![role("user")]);
        assert!(!caller.grants.allows("USERS_VIEW"));
    }

    #[tokio::test]
    async fn an_unknown_user_is_not_found() {
        let harness = Harness::new().await;
        let result = harness.service.get_user(crate::identity::domain::UserId(42)).await;
        assert!(matches!(result, Err(IdentityError::UserNotFound)));
    }
}
