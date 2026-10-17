use crate::identity::domain::{IdentityError, KeycloakId, RoleName};

/// Keycloak's admin API, as managing roles and accounts needs it: realm roles
/// are what a token carries, so a role made here exists there too, and a role
/// is given to a person there. A person's token shows the change once it is
/// refreshed.
pub trait RealmAdmin: Clone + Send + Sync + 'static {
    /// Makes the realm role; one that already exists is left as it is.
    fn create_role(&self, name: &RoleName, description: &str)
    -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Removes the realm role; one that is already gone is no error.
    fn delete_role(&self, name: &RoleName) -> impl Future<Output = Result<(), IdentityError>> + Send;

    fn assign_role(
        &self,
        account: &KeycloakId,
        name: &RoleName,
    ) -> impl Future<Output = Result<(), IdentityError>> + Send;

    fn remove_role(
        &self,
        account: &KeycloakId,
        name: &RoleName,
    ) -> impl Future<Output = Result<(), IdentityError>> + Send;

    /// Stores the language Keycloak shows its pages to the account in, which
    /// for a signed-in person comes before the language a sign-in asks for.
    fn set_language(
        &self,
        account: &KeycloakId,
        language: &str,
    ) -> impl Future<Output = Result<(), IdentityError>> + Send;
}
