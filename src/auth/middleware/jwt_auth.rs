use axum::{
	extract::{Request, State},
	middleware::Next,
	response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::auth::services::auth_service::Claims;
use crate::common::error::error::AppError;
use crate::i18n::setup::translate;

pub async fn auth_middleware(
	State(jwt_access_token_secret): State<String>,
	mut request: Request,
	next: Next,
) -> Result<Response, AppError> {
	let token = request
		.headers()
		.get("Authorization")
		.and_then(|auth_header| auth_header.to_str().ok())
		.and_then(|auth_value| {
			if auth_value.starts_with("Bearer ") {
				Some(auth_value[7..].to_owned())
			} else {
				None
			}
		})
		.ok_or(AppError::AuthenticationError(translate("auth.errors.missing_token")))?;

	let token_data = decode::<Claims>(
		&token,
		&DecodingKey::from_secret(jwt_access_token_secret.as_bytes()),
		&Validation::default(),
	)
	.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_token")))?;

	request.extensions_mut().insert(token_data.claims);

	Ok(next.run(request).await)
}
