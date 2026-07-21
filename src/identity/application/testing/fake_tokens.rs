use jiff::Timestamp;
use uuid::Uuid;

use crate::identity::application::ports::TokenService;
use crate::identity::application::token::{AccessClaims, ConfirmationClaims, TokenKind};
use crate::identity::domain::{Email, IdentityError, RoleName, User, UserId};

/// Transparent tokens: pipe-separated payloads instead of real JWTs, so
/// tests can inspect and craft tokens freely.
#[derive(Clone)]
pub(crate) struct FakeTokens {
	pub ttl: i64,
}

impl Default for FakeTokens {
	fn default() -> Self {
		Self { ttl: 3600 }
	}
}

fn kind_str(kind: TokenKind) -> &'static str {
	match kind {
		TokenKind::EmailConfirmation => "email_confirmation",
		TokenKind::EmailChange => "email_change",
		TokenKind::PasswordReset => "password_reset",
	}
}

fn parse_kind(value: &str) -> Result<TokenKind, IdentityError> {
	match value {
		"email_confirmation" => Ok(TokenKind::EmailConfirmation),
		"email_change" => Ok(TokenKind::EmailChange),
		"password_reset" => Ok(TokenKind::PasswordReset),
		_ => Err(IdentityError::InvalidToken),
	}
}

impl TokenService for FakeTokens {
	fn sign_access(&self, user: &User) -> Result<String, IdentityError> {
		let roles = user.roles.iter().map(|role| role.as_str()).collect::<Vec<_>>().join(",");
		Ok(format!("access|{}|{}|{}|{roles}", user.id.0, user.username, user.email))
	}

	fn verify_access(&self, token: &str) -> Result<AccessClaims, IdentityError> {
		let mut parts = token.split('|');
		if parts.next() != Some("access") {
			return Err(IdentityError::InvalidToken);
		}
		let sub = parts.next().and_then(|v| v.parse().ok()).ok_or(IdentityError::InvalidToken)?;
		let username = parts.next().ok_or(IdentityError::InvalidToken)?.to_string();
		let email = parts.next().ok_or(IdentityError::InvalidToken)?.to_string();
		let roles = parts
			.next()
			.unwrap_or_default()
			.split(',')
			.filter_map(|role| role.parse::<RoleName>().ok())
			.collect();
		let now = Timestamp::now().as_second();
		Ok(AccessClaims { sub, username, email, roles, exp: now + self.ttl, iat: now })
	}

	fn confirmation_ttl_seconds(&self) -> i64 {
		self.ttl
	}

	fn sign_confirmation(
		&self,
		kind: TokenKind,
		user_id: UserId,
		email: &Email,
		new_email: Option<&Email>,
	) -> Result<String, IdentityError> {
		Ok(format!(
			"confirm|{}|{}|{}|{}|{}",
			kind_str(kind),
			user_id.0,
			email,
			new_email.map(ToString::to_string).unwrap_or_default(),
			Uuid::new_v4()
		))
	}

	fn verify_confirmation(&self, token: &str) -> Result<ConfirmationClaims, IdentityError> {
		let mut parts = token.split('|');
		if parts.next() != Some("confirm") {
			return Err(IdentityError::InvalidToken);
		}
		let kind = parse_kind(parts.next().ok_or(IdentityError::InvalidToken)?)?;
		let sub = parts.next().and_then(|v| v.parse().ok()).ok_or(IdentityError::InvalidToken)?;
		let email = parts.next().ok_or(IdentityError::InvalidToken)?.to_string();
		let new_email = parts.next().filter(|v| !v.is_empty()).map(str::to_string);
		let jti = parts.next().ok_or(IdentityError::InvalidToken)?.to_string();
		let now = Timestamp::now().as_second();
		Ok(ConfirmationClaims { sub, email, kind, new_email, exp: now + self.ttl, iat: now, jti })
	}
}
