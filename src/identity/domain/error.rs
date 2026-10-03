use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum IdentityError {
	#[error("invalid email address")]
	InvalidEmail,
	#[error("missing bearer token or session")]
	Unauthenticated,
	#[error("invalid or unknown token")]
	InvalidToken,
	#[error("admin role required")]
	AdminRoleRequired,
	#[error("user not found")]
	UserNotFound,
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
