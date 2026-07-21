use thiserror::Error;

/// All failures the identity module can produce. Adapters reduce their
/// technology errors to the `*Failure` variants; only the HTTP adapter
/// turns this enum into responses.
#[derive(Debug, Error)]
pub enum IdentityError {
	#[error("invalid email address")]
	InvalidEmail,
	#[error("username is too short")]
	UsernameTooShort,
	#[error("email address already in use")]
	EmailTaken,
	#[error("username already in use")]
	UsernameTaken,
	#[error("invalid credentials")]
	InvalidCredentials,
	#[error("account is inactive")]
	AccountInactive,
	#[error("email address is not confirmed")]
	EmailNotConfirmed,
	#[error("email address is already confirmed")]
	EmailAlreadyConfirmed,
	#[error("new email is the same as the current one")]
	SameEmailAsCurrent,
	#[error("invalid current password")]
	InvalidCurrentPassword,
	#[error("missing bearer token")]
	MissingBearerToken,
	#[error("admin role required")]
	AdminRoleRequired,
	#[error("invalid or unknown token")]
	InvalidToken,
	#[error("token has expired")]
	ExpiredToken,
	#[error("invalid token type")]
	InvalidTokenType,
	#[error("user not found")]
	UserNotFound,

	#[error("missing refresh token")]
	RefreshTokenMissing,
	#[error("invalid refresh token")]
	RefreshTokenInvalid,
	#[error("refresh token has expired")]
	RefreshTokenExpired,

	#[error("persistence failure: {0}")]
	PersistenceFailure(String),
	#[error("stored data is corrupt: {0}")]
	CorruptData(String),
	#[error("mailer failure: {0}")]
	MailerFailure(String),
	#[error("password hashing failure")]
	HashingFailure,
	#[error("token signing failure")]
	TokenSigningFailure,
}
