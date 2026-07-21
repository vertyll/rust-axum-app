//! HTTP authentication. [`authenticate`] is a generic middleware layered by
//! the composition root onto protected routers (also of other modules);
//! [`Auth`]/[`RequireAdmin`] read the claims it stores in the request
//! extensions — that is the whole surface other modules need.

use std::sync::Arc;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::header;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;

use super::error::ApiError;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::application::token::AccessClaims;
use crate::identity::domain::{IdentityError, RoleName};

/// Middleware guarding a router: requires a valid bearer token bound to an
/// active, confirmed account.
pub async fn authenticate<P: IdentityPorts>(
	State(identity): State<Arc<IdentityService<P>>>,
	mut request: Request,
	next: Next,
) -> Result<Response, ApiError> {
	let token = bearer_token(request.headers()).ok_or(IdentityError::MissingBearerToken)?;
	let claims = identity.authenticate(&token).await?;
	request.extensions_mut().insert(claims);
	Ok(next.run(request).await)
}

fn bearer_token(headers: &axum::http::HeaderMap) -> Option<String> {
	headers
		.get(header::AUTHORIZATION)?
		.to_str()
		.ok()?
		.strip_prefix("Bearer ")
		.map(str::to_owned)
}

/// The authenticated caller's claims. Requires the [`authenticate`] layer.
pub struct Auth(pub AccessClaims);

impl<S: Send + Sync> FromRequestParts<S> for Auth {
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		parts
			.extensions
			.get::<AccessClaims>()
			.cloned()
			.map(Auth)
			.ok_or_else(|| IdentityError::MissingBearerToken.into())
	}
}

/// Like [`Auth`], but additionally requires the `admin` role.
pub struct RequireAdmin(pub AccessClaims);

impl<S: Send + Sync> FromRequestParts<S> for RequireAdmin {
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let Auth(claims) = Auth::from_request_parts(parts, state).await?;
		if !claims.has_role(RoleName::Admin) {
			return Err(IdentityError::AdminRoleRequired.into());
		}
		Ok(RequireAdmin(claims))
	}
}
#[cfg(test)]
mod tests {
	use axum::http::{HeaderMap, HeaderValue, header};

	use super::bearer_token;

	#[test]
	fn extracts_bearer_value() {
		let mut headers = HeaderMap::new();
		headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer abc.def"));
		assert_eq!(bearer_token(&headers).as_deref(), Some("abc.def"));
	}

	#[test]
	fn ignores_absence_and_other_schemes() {
		let mut headers = HeaderMap::new();
		assert!(bearer_token(&headers).is_none());
		headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Basic abc"));
		assert!(bearer_token(&headers).is_none());
	}
}
