//! JWT adapter for the `TokenService` port (HS256 via `jsonwebtoken`).

use jiff::Timestamp;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use uuid::Uuid;

use crate::identity::application::ports::TokenService;
use crate::identity::application::token::{AccessClaims, ConfirmationClaims, TokenKind};
use crate::identity::domain::{Email, IdentityError, User, UserId};

#[derive(Clone)]
pub struct JwtTokenService {
	access_secret: String,
	access_ttl_seconds: i64,
	confirmation_secret: String,
	confirmation_ttl_seconds: i64,
}

impl JwtTokenService {
	pub fn new(
		access_secret: String,
		access_ttl_seconds: i64,
		confirmation_secret: String,
		confirmation_ttl_seconds: i64,
	) -> Self {
		Self { access_secret, access_ttl_seconds, confirmation_secret, confirmation_ttl_seconds }
	}

	fn sign<C: serde::Serialize>(&self, claims: &C, secret: &str) -> Result<String, IdentityError> {
		encode(&Header::default(), claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(
			|err| {
				tracing::error!("token signing failed: {err}");
				IdentityError::TokenSigningFailure
			},
		)
	}
}

impl TokenService for JwtTokenService {
	fn sign_access(&self, user: &User) -> Result<String, IdentityError> {
		let now = Timestamp::now().as_second();
		let claims = AccessClaims {
			sub: user.id.0,
			username: user.username.to_string(),
			email: user.email.to_string(),
			roles: user.roles.clone(),
			iat: now,
			exp: now + self.access_ttl_seconds,
		};
		self.sign(&claims, &self.access_secret)
	}

	fn verify_access(&self, token: &str) -> Result<AccessClaims, IdentityError> {
		decode::<AccessClaims>(
			token,
			&DecodingKey::from_secret(self.access_secret.as_bytes()),
			&Validation::default(),
		)
		.map(|data| data.claims)
		.map_err(|_| IdentityError::InvalidToken)
	}

	fn confirmation_ttl_seconds(&self) -> i64 {
		self.confirmation_ttl_seconds
	}

	fn sign_confirmation(
		&self,
		kind: TokenKind,
		user_id: UserId,
		email: &Email,
		new_email: Option<&Email>,
	) -> Result<String, IdentityError> {
		let now = Timestamp::now().as_second();
		let claims = ConfirmationClaims {
			sub: user_id.0,
			email: email.to_string(),
			kind,
			new_email: new_email.map(ToString::to_string),
			iat: now,
			exp: now + self.confirmation_ttl_seconds,
			jti: Uuid::new_v4().to_string(),
		};
		self.sign(&claims, &self.confirmation_secret)
	}

	fn verify_confirmation(&self, token: &str) -> Result<ConfirmationClaims, IdentityError> {
		decode::<ConfirmationClaims>(
			token,
			&DecodingKey::from_secret(self.confirmation_secret.as_bytes()),
			&Validation::default(),
		)
		.map(|data| data.claims)
		.map_err(|_| IdentityError::InvalidToken)
	}
}
#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use super::JwtTokenService;
	use crate::identity::application::ports::TokenService;
	use crate::identity::application::token::TokenKind;
	use crate::identity::domain::{
		Email, IdentityError, PasswordHash, RoleName, User, UserId, Username,
	};

	fn service(ttl: i64) -> JwtTokenService {
		JwtTokenService::new("access-secret".into(), ttl, "confirm-secret".into(), ttl)
	}

	fn user() -> User {
		User {
			id: UserId(7),
			username: Username::parse("alice").unwrap(),
			email: Email::parse("alice@example.com").unwrap(),
			password_hash: PasswordHash::new("hash"),
			is_email_confirmed: true,
			is_active: true,
			email_confirmation: None,
			password_reset: None,
			email_change: None,
			roles: vec![RoleName::Admin, RoleName::User],
			created_at: Timestamp::UNIX_EPOCH,
			updated_at: Timestamp::UNIX_EPOCH,
		}
	}

	#[test]
	fn access_token_round_trip() {
		let service = service(3600);
		let claims = service.verify_access(&service.sign_access(&user()).unwrap()).unwrap();
		assert_eq!(claims.sub, 7);
		assert_eq!(claims.username, "alice");
		assert_eq!(claims.roles, vec![RoleName::Admin, RoleName::User]);
	}

	#[test]
	fn rejects_foreign_and_expired_tokens() {
		let token = service(3600).sign_access(&user()).unwrap();
		let foreign =
			JwtTokenService::new("other".into(), 3600, "confirm-secret".into(), 3600);
		assert!(matches!(foreign.verify_access(&token), Err(IdentityError::InvalidToken)));

		let stale = service(-3600).sign_access(&user()).unwrap();
		assert!(matches!(service(3600).verify_access(&stale), Err(IdentityError::InvalidToken)));
	}

	#[test]
	fn confirmation_token_preserves_kind_and_new_email() {
		let service = service(3600);
		let new_email = Email::parse("new@example.com").unwrap();
		let token = service
			.sign_confirmation(TokenKind::EmailChange, UserId(7), &user().email, Some(&new_email))
			.unwrap();

		let claims = service.verify_confirmation(&token).unwrap();
		assert_eq!(claims.kind, TokenKind::EmailChange);
		assert_eq!(claims.user_id(), UserId(7));
		assert_eq!(claims.new_email.as_deref(), Some("new@example.com"));
	}
}
