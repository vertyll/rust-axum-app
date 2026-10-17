/// The second factor a person set up on Keycloak's pages: the id of its
/// credential, which Keycloak's `delete_credential` action takes to remove it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecondFactor {
    pub credential_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum KeycloakAccountError {
    #[error("Keycloak's account API is unavailable: {0}")]
    Unavailable(String),
}
