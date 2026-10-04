use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::OnceCell;

use super::{KeycloakSettings, SharedRefreshes};
use crate::identity::application::ports::KeycloakClient;
use crate::identity::application::session::TokenPair;
use crate::identity::domain::IdentityError;

const TIMEOUT: Duration = Duration::from_secs(10);
const REUSE_WINDOW: Duration = Duration::from_secs(30);

type SharedRefresh = Arc<OnceCell<Result<TokenPair, IdentityError>>>;

/// Keycloak's token endpoint over HTTP. A refresh is run once per refresh
/// token: concurrent callers wait for it, and for thirty seconds a caller
/// still holding the old token receives the same result, so a rotated
/// refresh token is never presented twice. Replicas agree on the same
/// refresh through [`SharedRefreshes`].
#[derive(Clone)]
pub struct HttpKeycloakClient {
	http: reqwest::Client,
	settings: Arc<KeycloakSettings>,
	refreshes: Arc<Mutex<HashMap<String, (Instant, SharedRefresh)>>>,
	shared: SharedRefreshes,
}

impl HttpKeycloakClient {
	pub fn new(settings: KeycloakSettings, shared: SharedRefreshes) -> Result<Self, IdentityError> {
		let http = reqwest::Client::builder()
			.timeout(TIMEOUT)
			.build()
			.map_err(|err| IdentityError::IdentityProviderUnavailable(err.to_string()))?;
		Ok(Self {
			http,
			settings: Arc::new(settings),
			refreshes: Arc::default(),
			shared,
		})
	}

	fn shared_refresh(&self, refresh_token: &str) -> SharedRefresh {
		let mut refreshes = self.refreshes.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
		refreshes.retain(|_, (started, cell)| !(cell.initialized() && started.elapsed() > REUSE_WINDOW));
		refreshes
			.entry(refresh_token.to_string())
			.or_insert_with(|| (Instant::now(), Arc::new(OnceCell::new())))
			.1
			.clone()
	}

	fn forget(&self, refresh_token: &str, cell: &SharedRefresh) {
		let mut refreshes = self.refreshes.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
		if refreshes
			.get(refresh_token)
			.is_some_and(|(_, stored)| Arc::ptr_eq(stored, cell))
		{
			refreshes.remove(refresh_token);
		}
	}

	async fn token(&self, form: &[(&str, &str)], on_rejection: IdentityError) -> Result<TokenPair, IdentityError> {
		let response = self
			.http
			.post(self.settings.endpoint("token"))
			.form(&self.with_client(form))
			.send()
			.await
			.map_err(unavailable)?;
		let status = response.status();
		if status.is_server_error() {
			return Err(IdentityError::IdentityProviderUnavailable(status.to_string()));
		}
		if status.is_client_error() {
			return Err(on_rejection);
		}
		let body: TokenResponse = response.json().await.map_err(unavailable)?;
		match (body.access_token, body.refresh_token) {
			(Some(access_token), Some(refresh_token)) => Ok(TokenPair {
				access_token,
				refresh_token,
			}),
			_ => Err(IdentityError::IdentityProviderUnavailable(
				"incomplete token response".into(),
			)),
		}
	}

	fn with_client<'a>(&'a self, form: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
		let mut fields = vec![
			("client_id", self.settings.client_id.as_str()),
			("client_secret", self.settings.client_secret.as_str()),
		];
		fields.extend_from_slice(form);
		fields
	}
}

impl KeycloakClient for HttpKeycloakClient {
	async fn exchange(&self, code: &str, code_verifier: &str) -> Result<TokenPair, IdentityError> {
		let callback_url = self.settings.callback_url.clone();
		self.token(
			&[
				("grant_type", "authorization_code"),
				("code", code),
				("code_verifier", code_verifier),
				("redirect_uri", &callback_url),
			],
			IdentityError::SignInRejected,
		)
		.await
	}

	async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, IdentityError> {
		let cell = self.shared_refresh(refresh_token);
		let result = cell
			.get_or_init(|| async move {
				let form = [("grant_type", "refresh_token"), ("refresh_token", refresh_token)];
				self.shared
					.refresh(refresh_token, || self.token(&form, IdentityError::SessionExpired))
					.await
			})
			.await
			.clone();
		if result.is_err() {
			self.forget(refresh_token, &cell);
		}
		result
	}

	async fn revoke(&self, refresh_token: &str) {
		let result = self
			.http
			.post(self.settings.endpoint("logout"))
			.form(&self.with_client(&[("refresh_token", refresh_token)]))
			.send()
			.await;
		match result {
			Ok(response) if response.status().is_success() => {}
			Ok(response) => tracing::warn!(status = %response.status(), "Keycloak did not end the session"),
			Err(err) => tracing::warn!("Keycloak did not end the session: {err}"),
		}
	}
}

#[derive(Deserialize)]
struct TokenResponse {
	access_token: Option<String>,
	refresh_token: Option<String>,
}

fn unavailable(err: reqwest::Error) -> IdentityError {
	IdentityError::IdentityProviderUnavailable(err.to_string())
}

#[cfg(test)]
mod tests {
	use std::sync::Arc;
	use std::sync::atomic::{AtomicUsize, Ordering};
	use std::time::Duration;

	use axum::Router;
	use axum::http::StatusCode;
	use axum::routing::post;

	use super::HttpKeycloakClient;
	use crate::identity::application::ports::KeycloakClient;
	use crate::identity::domain::IdentityError;
	use crate::identity::infrastructure::keycloak::{KeycloakSettings, SharedRefreshes};

	async fn keycloak(status: StatusCode, delay: Duration) -> (HttpKeycloakClient, Arc<AtomicUsize>) {
		let calls = Arc::new(AtomicUsize::new(0));
		let counter = calls.clone();
		let app = Router::new().route(
			"/realms/test/protocol/openid-connect/token",
			post(move || {
				let counter = counter.clone();
				async move {
					counter.fetch_add(1, Ordering::SeqCst);
					tokio::time::sleep(delay).await;
					(status, r#"{"access_token":"access-2","refresh_token":"refresh-2"}"#)
				}
			}),
		);
		let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
		let address = listener.local_addr().unwrap();
		tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });

		let client = HttpKeycloakClient::new(
			KeycloakSettings {
				realm_url: format!("http://{address}/realms/test"),
				client_id: "client".into(),
				client_secret: "secret".into(),
				audience: "client".into(),
				callback_url: "http://app.test/callback".into(),
			},
			SharedRefreshes::in_process_only(),
		)
		.unwrap();
		(client, calls)
	}

	#[tokio::test]
	async fn concurrent_refreshes_of_one_token_reach_keycloak_once() {
		let (client, calls) = keycloak(StatusCode::OK, Duration::from_millis(200)).await;

		let (first, second) = tokio::join!(client.refresh("refresh-1"), client.refresh("refresh-1"));

		assert_eq!(first.unwrap().refresh_token, "refresh-2");
		assert_eq!(second.unwrap().refresh_token, "refresh-2");
		assert_eq!(calls.load(Ordering::SeqCst), 1);
	}

	#[tokio::test]
	async fn a_session_read_before_the_refresh_gets_the_tokens_already_issued() {
		let (client, calls) = keycloak(StatusCode::OK, Duration::ZERO).await;

		client.refresh("refresh-1").await.unwrap();
		let stale = client.refresh("refresh-1").await.unwrap();

		assert_eq!(stale.access_token, "access-2");
		assert_eq!(calls.load(Ordering::SeqCst), 1);
	}

	#[tokio::test]
	async fn a_refused_refresh_ends_the_session_and_is_not_remembered() {
		let (client, calls) = keycloak(StatusCode::BAD_REQUEST, Duration::ZERO).await;

		assert!(matches!(
			client.refresh("refresh-1").await,
			Err(IdentityError::SessionExpired)
		));
		assert!(matches!(
			client.refresh("refresh-1").await,
			Err(IdentityError::SessionExpired)
		));
		assert_eq!(calls.load(Ordering::SeqCst), 2);
	}

	#[tokio::test]
	async fn a_keycloak_outage_is_reported_as_such() {
		let (client, _) = keycloak(StatusCode::SERVICE_UNAVAILABLE, Duration::ZERO).await;

		assert!(matches!(
			client.exchange("code", "verifier").await,
			Err(IdentityError::IdentityProviderUnavailable(_))
		));
	}
}
