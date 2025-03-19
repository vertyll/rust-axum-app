use crate::auth::services::auth_service::Claims;
use crate::common::error::app_error::AppError;
use crate::di::DatabaseConnectionTrait;
use crate::i18n::setup::translate;
use crate::users::entities::users::Column;
use crate::users::entities::users::Entity as User;
use axum::{extract::FromRequestParts, http::request::Parts};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::future::Future;
use std::sync::Arc;

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

			let claims = token_data.claims;

			// Get database connection from request extensions
			let db = parts
				.extensions
				.get::<Arc<dyn DatabaseConnectionTrait>>()
				.ok_or_else(|| AppError::InternalError)?
				.get_connection();

			// Verify user is active and email confirmed
			let user = User::find()
				.filter(Column::Id.eq(claims.sub))
				.one(db)
				.await
				.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_user")))?
				.ok_or_else(|| AppError::AuthenticationError(translate("auth.errors.user_not_found")))?;

			if !user.is_active {
				return Err(AppError::AuthenticationError(translate("auth.errors.account_inactive")));
			}

			if !user.is_email_confirmed {
				return Err(AppError::AuthenticationError(translate(
					"auth.errors.email_not_confirmed",
				)));
			}

			Ok(JwtAuth(claims))
		}
	}
}
