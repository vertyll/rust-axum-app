use crate::keycloak_account::domain::{KeycloakAccountError, SecondFactor};
use crate::translations::Language;

/// Keycloak's account API, called with the person's own access token, which
/// answers about that person alone.
pub trait AccountApi: Clone + Send + Sync + 'static {
    /// The person's second factor, if they set one up.
    fn second_factor(
        &self,
        access_token: &str,
    ) -> impl Future<Output = Result<Option<SecondFactor>, KeycloakAccountError>> + Send;

    /// Stores the language Keycloak shows its pages to the person in, which for
    /// a signed-in person comes before the language a sign-in asks for.
    fn set_language(
        &self,
        access_token: &str,
        language: Language,
    ) -> impl Future<Output = Result<(), KeycloakAccountError>> + Send;
}
