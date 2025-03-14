use axum::{
	extract::{FromRequestParts, FromRef},
	http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::auth::services::auth_service::Claims;
use crate::i18n::setup::translate;
use std::future::Future;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;

pub struct JwtAuth(pub Claims);

impl<S> FromRequestParts<S> for JwtAuth
where
	AppState: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = AppError;

	fn from_request_parts(
		parts: &mut Parts,
		state: &S,
	) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			let app_state = AppState::from_ref(state);

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
				&DecodingKey::from_secret(app_state.jwt_access_token_secret.as_bytes()),
				&Validation::default(),
			)
				.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_token")))?;

			Ok(JwtAuth(token_data.claims))
		}
	}
}
