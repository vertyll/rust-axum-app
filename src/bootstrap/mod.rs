//! Composition root — the only place naming concrete adapters. Dependency
//! injection is constructor calls plus the aliases below, all resolved at
//! compile time; nothing imports this module.

pub mod config;
pub mod seed;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};
use tower_sessions_redis_store::RedisStore;
use tower_sessions_redis_store::fred::prelude::{ClientLike, Config as RedisConfig, Pool as RedisPool};

use crate::files::application::FilesService;
use crate::files::infrastructure::http as files_http;
use crate::files::infrastructure::persistence::ToastyFileRepository;
use crate::files::infrastructure::storage::LocalFileStorage;
use crate::identity::application::IdentityService;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::infrastructure::http as identity_http;
use crate::identity::infrastructure::keycloak::{HttpKeycloakClient, JwksTokenVerifier, KeycloakSettings};
use crate::identity::infrastructure::persistence::ToastyUserRepository;
use crate::shared_infrastructure::problem;
use crate::translations::application::TranslationsService;
use crate::translations::infrastructure::defaults;
use crate::translations::infrastructure::http as translations_http;
use crate::translations::infrastructure::persistence::ToastyTranslationRepository;
use config::AppConfig;

/// A pure type-level bundle: it only names which adapter satisfies each
/// port. Adding a port means one associated type here — nothing else.
#[derive(Clone)]
pub struct ProductionPorts;

impl IdentityPorts for ProductionPorts {
	type Users = ToastyUserRepository;
	type Keycloak = HttpKeycloakClient;
	type Verifier = JwksTokenVerifier;
}

/// The identity service with the production adapters plugged in.
pub type AppIdentityService = IdentityService<ProductionPorts>;

/// The files service with the production adapters plugged in.
pub type AppFilesService = FilesService<ToastyFileRepository, LocalFileStorage>;

/// The translations service with the production adapters plugged in.
pub type AppTranslationsService = TranslationsService<ToastyTranslationRepository>;

pub struct Services {
	pub identity: Arc<AppIdentityService>,
	pub files: Arc<AppFilesService>,
	pub translations: Arc<AppTranslationsService>,
}

/// Wires concrete adapters into the application services.
pub fn build_services(db: toasty::Db, config: &AppConfig) -> anyhow::Result<Services> {
	let keycloak = keycloak_settings(config);

	let identity: AppIdentityService = IdentityService::new(
		ToastyUserRepository::new(db.clone()),
		HttpKeycloakClient::new(keycloak.clone())?,
		JwksTokenVerifier::new(&keycloak)?,
	);

	let files = FilesService::new(
		ToastyFileRepository::new(db.clone()),
		LocalFileStorage::new(&config.files.upload_dir, &config.files.base_url),
	);

	let translations = TranslationsService::new(ToastyTranslationRepository::new(db.clone()), defaults::shipped()?);

	Ok(Services {
		identity: Arc::new(identity),
		files: Arc::new(files),
		translations: Arc::new(translations),
	})
}

/// Connects the session store: browser sessions live in Redis, so the
/// application holds no state of its own between requests.
pub async fn session_store(config: &AppConfig) -> anyhow::Result<RedisStore<RedisPool>> {
	let pool = RedisPool::new(RedisConfig::from_url(&config.sessions.redis_url)?, None, None, None, 1)?;
	pool.init().await?;
	Ok(RedisStore::new(pool))
}

/// Assembles the HTTP router. The composition root decides which routers
/// sit behind the auth guard; the modules only provide their routes.
pub fn router(services: &Services, config: &AppConfig, store: RedisStore<RedisPool>) -> Router {
	let auth_layer = from_fn_with_state(
		services.identity.clone(),
		identity_http::authenticate::<ProductionPorts>,
	);

	let sign_in = identity_http::SignInSettings {
		keycloak: keycloak_settings(config),
		post_login_url: config.keycloak.post_login_url.clone(),
	};

	let sessions = SessionManagerLayer::new(store)
		.with_name("RUST_AXUM_APP_SESSION")
		.with_http_only(true)
		.with_same_site(SameSite::Lax)
		.with_secure(config.sessions.cookie_secure)
		.with_path("/")
		.with_expiry(Expiry::OnInactivity(Duration::seconds(
			config.sessions.inactivity_timeout_seconds,
		)));

	let users_routes = identity_http::users_router(services.identity.clone()).layer(auth_layer.clone());

	let files_routes = files_http::files_router(services.files.clone()).layer(auth_layer.clone());

	let translations_admin_routes =
		translations_http::translations_admin_router(services.translations.clone()).layer(auth_layer);

	Router::new()
		.nest(
			"/api/auth",
			identity_http::auth_router(services.identity.clone(), sign_in),
		)
		.nest("/api/users", users_routes)
		.nest("/api/files", files_routes)
		.nest(
			"/api/translations",
			translations_http::translations_public_router(services.translations.clone()),
		)
		.nest("/api/admin/translations", translations_admin_routes)
		.nest_service("/uploads", ServeDir::new(&config.files.upload_dir))
		.nest_service("/legal", ServeDir::new("resources/legal"))
		.route("/health", get(health))
		.fallback(problem::not_found)
		.layer(sessions)
		.layer(TraceLayer::new_for_http())
}

fn keycloak_settings(config: &AppConfig) -> KeycloakSettings {
	KeycloakSettings {
		realm_url: config.keycloak.realm_url.clone(),
		client_id: config.keycloak.client_id.clone(),
		client_secret: config.keycloak.client_secret.clone(),
		audience: config.keycloak.audience.clone(),
		callback_url: config.keycloak.callback_url.clone(),
	}
}

async fn health() -> Json<serde_json::Value> {
	Json(serde_json::json!({ "status": "UP" }))
}
