//! HTTP authentication. [`authenticate`] is a generic middleware layered by
//! the composition root onto protected routers (also of other modules); it
//! takes the access token from `Authorization: Bearer` or, for a browser,
//! from its session, refreshing it when it is about to expire. [`Auth`] and
//! [`RequireAdmin`] read the [`Caller`] it stores in the request extensions.

use std::sync::Arc;

use axum::extract::{FromRequestParts, Request, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, Method, header};
use axum::middleware::Next;
use axum::response::Response;
use jiff::{SignedDuration, Timestamp};
use tower_sessions::Session;

use super::error::ApiError;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::application::{AuthSession, Caller};
use crate::identity::domain::{IdentityError, RoleName};

pub(crate) const SESSION_KEY: &str = "identity.session";
const REFRESH_SKEW: SignedDuration = SignedDuration::from_secs(30);
const FETCH_SITE: &str = "sec-fetch-site";

pub async fn authenticate<P: IdentityPorts>(
	State(identity): State<Arc<IdentityService<P>>>,
	session: Session,
	mut request: Request,
	next: Next,
) -> Result<Response, ApiError> {
	let token = match bearer_token(request.headers()) {
		Some(token) => token,
		None if sent_by_same_origin(request.method(), request.headers()) => session_token(&identity, &session)
			.await?
			.ok_or(IdentityError::Unauthenticated)?,
		None => return Err(IdentityError::Unauthenticated.into()),
	};
	let caller = identity.authenticate(&token).await?;
	request.extensions_mut().insert(caller);
	Ok(next.run(request).await)
}

async fn session_token<P: IdentityPorts>(
	identity: &IdentityService<P>,
	session: &Session,
) -> Result<Option<String>, IdentityError> {
	let Some(current) = session.get::<AuthSession>(SESSION_KEY).await.map_err(store)? else {
		return Ok(None);
	};
	if !current.needs_refresh_at(Timestamp::now(), REFRESH_SKEW) {
		return Ok(Some(current.access_token));
	}
	match identity.refresh(&current).await {
		Ok(refreshed) => {
			session.insert(SESSION_KEY, &refreshed).await.map_err(store)?;
			Ok(Some(refreshed.access_token))
		}
		Err(IdentityError::SessionExpired) => {
			session.flush().await.map_err(store)?;
			Ok(None)
		}
		Err(err) => {
			tracing::warn!(keycloak_id = %current.identity.keycloak_id, "could not refresh the session: {err}");
			Ok(None)
		}
	}
}

/// A cross-site write is not given the session's token: `SameSite=Lax` keeps
/// the cookie off cross-site requests, and this also covers sibling subdomains.
fn sent_by_same_origin(method: &Method, headers: &HeaderMap) -> bool {
	if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
		return true;
	}
	headers
		.get(FETCH_SITE)
		.and_then(|value| value.to_str().ok())
		.is_none_or(|site| site == "same-origin" || site == "none")
}

fn bearer_token(headers: &HeaderMap) -> Option<String> {
	headers
		.get(header::AUTHORIZATION)?
		.to_str()
		.ok()?
		.strip_prefix("Bearer ")
		.map(str::to_owned)
}

pub(crate) fn store(err: tower_sessions::session::Error) -> IdentityError {
	IdentityError::SessionStoreFailure(err.to_string())
}

/// The authenticated caller. Requires the [`authenticate`] layer.
pub struct Auth(pub Caller);

impl<S: Send + Sync> FromRequestParts<S> for Auth {
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
		parts
			.extensions
			.get::<Caller>()
			.cloned()
			.map(Auth)
			.ok_or_else(|| IdentityError::Unauthenticated.into())
	}
}

/// Like [`Auth`], but additionally requires the `admin` role.
pub struct RequireAdmin(pub Caller);

impl<S: Send + Sync> FromRequestParts<S> for RequireAdmin {
	type Rejection = ApiError;

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let Auth(caller) = Auth::from_request_parts(parts, state).await?;
		if !caller.has_role(RoleName::Admin) {
			return Err(IdentityError::AdminRoleRequired.into());
		}
		Ok(RequireAdmin(caller))
	}
}

#[cfg(test)]
mod tests {
	use axum::http::{HeaderMap, HeaderValue, Method, header};

	use super::{bearer_token, sent_by_same_origin};

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

	#[test]
	fn only_cross_site_writes_lose_the_session_token() {
		let mut cross_site = HeaderMap::new();
		cross_site.insert("sec-fetch-site", HeaderValue::from_static("cross-site"));
		assert!(sent_by_same_origin(&Method::GET, &cross_site));
		assert!(!sent_by_same_origin(&Method::POST, &cross_site));
		assert!(sent_by_same_origin(&Method::POST, &HeaderMap::new()));
	}
}
