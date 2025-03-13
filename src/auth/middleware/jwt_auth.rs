use axum::{
	extract::{Request, State},
	middleware::Next,
	response::Response,
};
use jsonwebtoken::{DecodingKey, Validation, decode};

use crate::auth::services::auth_service::Claims;
use crate::common::error::AppError;

pub async fn auth_middleware(
	State(jwt_secret): State<String>,
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
		.ok_or(AppError::AuthenticationError(
			"Missing or invalid authorization token".to_string(),
		))?;

	let token_data = decode::<Claims>(
		&token,
		&DecodingKey::from_secret(jwt_secret.as_bytes()),
		&Validation::default(),
	)
	.map_err(|_| AppError::AuthenticationError("Invalid token".to_string()))?;

	request.extensions_mut().insert(token_data.claims);

	Ok(next.run(request).await)
}
