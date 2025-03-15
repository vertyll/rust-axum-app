use crate::auth::services::auth_service::Claims;
use crate::common::error::app_error::AppError;
use crate::i18n::setup::translate;
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use std::future::Future;

#[derive(Clone)]
pub struct JwtSecret(pub String);

pub struct JwtAuth(pub Claims);

impl<S> FromRequestParts<S> for JwtAuth
where
	S: Send + Sync,
{
	type Rejection = AppError;

	fn from_request_parts(parts: &mut Parts, _state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			// Get JWT secret from extensions (added by middleware)
			let jwt_secret = parts
				.extensions
				.get::<JwtSecret>()
				.ok_or_else(|| AppError::InternalError)?;

			let token = parts
				.headers
				.get("Authorization")
				.and_then(|auth_header| auth_header.to_str().ok())
				.and_then(|auth_value| {
					if auth_value.starts_with("Bearer ") {
						Some(auth_value[7..].to_owned())
					} else {
						None
					}
				})
				.ok_or_else(|| AppError::AuthenticationError(translate("auth.errors.missing_token")))?;

			let token_data = decode::<Claims>(
				&token,
				&DecodingKey::from_secret(jwt_secret.0.as_bytes()),
				&Validation::default(),
			)
			.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_token")))?;

			Ok(JwtAuth(token_data.claims))
		}
	}
}
