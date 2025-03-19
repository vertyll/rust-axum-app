use crate::common::error::app_error::AppError;
use crate::i18n::setup::translate;
use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::di::AppConfigTrait;

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
	EmailConfirmation,
	EmailChange,
	PasswordReset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationClaims {
	pub sub: i32,
	pub email: String,
	pub token_type: TokenType,
	pub new_email: Option<String>,
	pub exp: i64,
	pub iat: i64,
	pub jti: String,
}

#[derive(Clone)]
pub struct ConfirmationTokenService {
	app_config: Arc<dyn AppConfigTrait>,
	confirmation_token_secret: String,
	confirmation_token_expires_in: i64,
}

impl ConfirmationTokenService {
	pub fn new(app_config: Arc<dyn AppConfigTrait>) -> Self {
		let confirmation_token_secret = app_config.get_config().security.tokens.confirmation_token.secret.clone();
		let confirmation_token_expires_in = app_config.get_config().security.tokens.confirmation_token.expires_in;
		Self {
			app_config,
			confirmation_token_secret,
			confirmation_token_expires_in
		}
	}
}

#[async_trait]
pub trait ConfirmationTokenServiceTrait: Send + Sync {
	async fn generate_email_confirmation_token(&self, user_id: i32, email: &str) -> Result<String, AppError>;
	async fn generate_email_change_token(
		&self,
		user_id: i32,
		current_email: &str,
		new_email: &str,
	) -> Result<String, AppError>;
	async fn generate_password_reset_token(&self, user_id: i32, email: &str) -> Result<String, AppError>;
	async fn validate_token(&self, token: &str) -> Result<ConfirmationClaims, AppError>;
	async fn validate_stored_token(
		&self,
		token: &str,
		stored_token: Option<&str>,
		expiry: Option<DateTime<Utc>>,
		expected_type: TokenType,
	) -> Result<ConfirmationClaims, AppError>;
}

#[async_trait]
impl ConfirmationTokenServiceTrait for ConfirmationTokenService {
	async fn generate_email_confirmation_token(&self, user_id: i32, email: &str) -> Result<String, AppError> {
		let now = Utc::now();
		let expires_at = now + Duration::seconds(self.confirmation_token_expires_in);

		let claims = ConfirmationClaims {
			sub: user_id,
			email: email.to_string(),
			token_type: TokenType::EmailConfirmation,
			new_email: None,
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
			jti: Uuid::new_v4().to_string(),
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(self.confirmation_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)?;

		Ok(token)
	}

	async fn generate_email_change_token(
		&self,
		user_id: i32,
		current_email: &str,
		new_email: &str,
	) -> Result<String, AppError> {
		let now = Utc::now();
		let expires_at = now + Duration::seconds(self.confirmation_token_expires_in);

		let claims = ConfirmationClaims {
			sub: user_id,
			email: current_email.to_string(),
			token_type: TokenType::EmailChange,
			new_email: Some(new_email.to_string()),
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
			jti: Uuid::new_v4().to_string(),
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(self.confirmation_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)?;

		Ok(token)
	}

	async fn generate_password_reset_token(&self, user_id: i32, email: &str) -> Result<String, AppError> {
		let now = Utc::now();
		let expires_at = now + Duration::seconds(self.confirmation_token_expires_in);

		let claims = ConfirmationClaims {
			sub: user_id,
			email: email.to_string(),
			token_type: TokenType::PasswordReset,
			new_email: None,
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
			jti: Uuid::new_v4().to_string(),
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(self.confirmation_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)?;

		Ok(token)
	}

	async fn validate_token(&self, token: &str) -> Result<ConfirmationClaims, AppError> {
		let token_data = decode::<ConfirmationClaims>(
			token,
			&DecodingKey::from_secret(self.confirmation_token_secret.as_bytes()),
			&Validation::default(),
		)
		.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_token")))?;

		Ok(token_data.claims)
	}

	async fn validate_stored_token(
		&self,
		token: &str,
		stored_token: Option<&str>,
		expiry: Option<DateTime<Utc>>,
		expected_type: TokenType,
	) -> Result<ConfirmationClaims, AppError> {
		let claims = self.validate_token(token).await?;

		if !matches!(claims.token_type, ref t if std::mem::discriminant(t) == std::mem::discriminant(&expected_type)) {
			return Err(AppError::AuthorizationError(
				translate("auth.errors.invalid_token_type").into(),
			));
		}

		if stored_token != Some(token) {
			return Err(AppError::AuthorizationError(
				translate("auth.errors.invalid_token").into(),
			));
		}

		if let Some(expires_at) = expiry {
			if Utc::now() > expires_at {
				return Err(AppError::AuthorizationError(
					translate("auth.errors.expired_token").into(),
				));
			}
		}

		Ok(claims)
	}
}
