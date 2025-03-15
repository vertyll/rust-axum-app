use crate::auth::extractor::jwt_auth_extractor::JwtSecret;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn jwt_secret_middleware(jwt_secret: String, request: Request, next: Next) -> Response {
	let mut request = request;

	request.extensions_mut().insert(JwtSecret(jwt_secret));

	next.run(request).await
}
