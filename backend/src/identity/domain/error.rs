use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum IdentityError {
    #[error("invalid email address")]
    InvalidEmail,
    #[error("missing bearer token or session")]
    Unauthenticated,
    #[error("invalid or unknown token")]
    InvalidToken,
    #[error("permission {0} required")]
    PermissionDenied(&'static str),
    #[error("user not found")]
    UserNotFound,
    #[error("invalid role name")]
    InvalidRoleName,
    #[error("role not found")]
    RoleNotFound,
    #[error("a role of that name exists")]
    RoleAlreadyExists,
    #[error("no module declares the permission {0}")]
    UnknownPermission(String),
    #[error("the last unrestricted role cannot be restricted")]
    LastUnrestrictedRole,
    #[error("a system role cannot be removed")]
    SystemRole,
    #[error("an unrestricted role must be restricted before it is removed")]
    UnrestrictedRole,
    #[error("a role some account holds cannot be removed")]
    RoleStillAssigned,
    #[error("the last account holding an unrestricted role cannot lose it")]
    LastUnrestrictedHolder,
    #[error("the sign-in could not be completed")]
    SignInRejected,
    #[error("the session has ended")]
    SessionExpired,
    #[error("the identity provider is unavailable: {0}")]
    IdentityProviderUnavailable(String),
    #[error("session store failure: {0}")]
    SessionStoreFailure(String),
    #[error("persistence failure: {0}")]
    PersistenceFailure(String),
    #[error("stored data is corrupt: {0}")]
    CorruptData(String),
}
