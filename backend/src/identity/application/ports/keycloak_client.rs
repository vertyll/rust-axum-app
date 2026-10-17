use crate::identity::application::session::TokenPair;
use crate::identity::domain::IdentityError;

/// The second factor a person set up on Keycloak's pages: the id of its credential,
/// which Keycloak's `delete_credential` action takes to remove it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondFactor {
    pub credential_id: String,
}

/// Keycloak's token endpoint, as the sign-in flow needs it, and the account API a
/// signed-in person reads their own credentials from.
pub trait KeycloakClient: Clone + Send + Sync + 'static {
    /// Redeems an authorization code; `SignInRejected` when Keycloak refuses it.
    fn exchange(
        &self,
        code: &str,
        code_verifier: &str,
    ) -> impl Future<Output = Result<TokenPair, IdentityError>> + Send;

    /// Rotates a refresh token; `SessionExpired` when Keycloak refuses it.
    /// Concurrent calls with the same token share one request.
    fn refresh(&self, refresh_token: &str) -> impl Future<Output = Result<TokenPair, IdentityError>> + Send;

    /// Ends the Keycloak session behind a refresh token, best-effort.
    fn revoke(&self, refresh_token: &str) -> impl Future<Output = ()> + Send;

    /// The second factor of the person whose access token this is, read with that
    /// token, so the application needs no right to read other people's credentials.
    fn second_factor(
        &self,
        access_token: &str,
    ) -> impl Future<Output = Result<Option<SecondFactor>, IdentityError>> + Send;
}
