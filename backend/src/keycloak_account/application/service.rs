//! The signed-in person's Keycloak account: the second factor, and the
//! language Keycloak's pages show them in.

use crate::keycloak_account::application::ports::AccountApi;
use crate::keycloak_account::domain::{KeycloakAccountError, SecondFactor};
use crate::translations::contracts::Language;

#[derive(Clone)]
pub struct KeycloakAccountService<A: AccountApi> {
    account: A,
}

impl<A: AccountApi> KeycloakAccountService<A> {
    pub fn new(account: A) -> Self {
        Self { account }
    }

    /// The person's second factor, read from Keycloak with their own access token.
    pub async fn second_factor(&self, access_token: &str) -> Result<Option<SecondFactor>, KeycloakAccountError> {
        self.account.second_factor(access_token).await
    }

    /// Makes Keycloak show the person's account pages, such as setting up a
    /// second factor, in the language they use here. When Keycloak cannot
    /// store it, the pages open in the language the account already has, so
    /// the failure is only logged.
    pub async fn keep_language(&self, access_token: &str, language: Language) {
        if let Err(err) = self.account.set_language(access_token, language).await {
            tracing::warn!("could not store the account's language in Keycloak: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeycloakAccountService;
    use crate::keycloak_account::application::testing::FakeAccountApi;
    use crate::translations::contracts::Language;

    #[tokio::test]
    async fn the_second_factor_is_the_one_keycloak_holds() {
        let account = FakeAccountApi::default();
        let service = KeycloakAccountService::new(account.clone());
        assert_eq!(service.second_factor("access").await.unwrap(), None);

        account.set_second_factor("otp-1");

        let factor = service.second_factor("access").await.unwrap().unwrap();
        assert_eq!(factor.credential_id, "otp-1");
    }

    #[tokio::test]
    async fn the_account_pages_follow_the_language_used_here() {
        let account = FakeAccountApi::default();
        let service = KeycloakAccountService::new(account.clone());

        service.keep_language("access", Language::Pl).await;

        assert_eq!(account.languages(), vec!["access pl".to_owned()]);
    }

    #[tokio::test]
    async fn a_language_keycloak_does_not_store_leaves_the_account_pages_as_they_are() {
        let account = FakeAccountApi::default();
        account.fail();
        let service = KeycloakAccountService::new(account.clone());

        service.keep_language("access", Language::Pl).await;

        assert_eq!(account.languages(), Vec::<String>::new());
    }
}
