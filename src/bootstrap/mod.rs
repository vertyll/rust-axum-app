//! Composition root — the only place naming concrete adapters. Dependency
//! injection is constructor calls plus the aliases below, all resolved at
//! compile time; nothing imports this module.

pub mod config;
pub mod seed;

use std::sync::Arc;

use axum::Router;
use axum::middleware::{from_fn, from_fn_with_state};
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use crate::files::application::FilesService;
use crate::files::infrastructure::http as files_http;
use crate::files::infrastructure::persistence::ToastyFileRepository;
use crate::files::infrastructure::storage::LocalFileStorage;
use crate::identity::application::IdentityService;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::infrastructure::email::{SmtpIdentityMailer, SmtpSettings};
use crate::identity::infrastructure::http as identity_http;
use crate::identity::infrastructure::persistence::{ToastySessionRepository, ToastyUserRepository};
use crate::identity::infrastructure::security::{Argon2PasswordHasher, JwtTokenService};
use crate::shared_infrastructure;
use config::AppConfig;

/// A pure type-level bundle: it only names which adapter satisfies each
/// port. Adding a port means one associated type here — nothing else.
#[derive(Clone)]
pub struct ProductionPorts;

impl IdentityPorts for ProductionPorts {
	type Users = ToastyUserRepository;
	type Sessions = ToastySessionRepository;
	type Hasher = Argon2PasswordHasher;
	type Tokens = JwtTokenService;
	type Mailer = SmtpIdentityMailer;
}

/// The identity service with the production adapters plugged in.
pub type AppIdentityService = IdentityService<ProductionPorts>;

/// The files service with the production adapters plugged in.
pub type AppFilesService = FilesService<ToastyFileRepository, LocalFileStorage>;

pub struct Services {
	pub identity: Arc<AppIdentityService>,
	pub files: Arc<AppFilesService>,
}

/// Wires concrete adapters into the application services.
pub fn build_services(db: toasty::Db, config: &AppConfig) -> anyhow::Result<Services> {
	let tokens = JwtTokenService::new(
		config.security.access_token.secret.clone(),
		config.security.access_token.expires_in_seconds,
		config.security.confirmation_token.secret.clone(),
		config.security.confirmation_token.expires_in_seconds,
	);

	let mailer = SmtpIdentityMailer::new(SmtpSettings {
		host: &config.emails.smtp_host,
		port: config.emails.smtp_port,
		username: &config.emails.smtp_username,
		password: &config.emails.smtp_password,
		from: &config.emails.from,
		templates_dir: &config.emails.templates_dir,
		app_url: &config.server.url,
	})?;

	let identity: AppIdentityService = IdentityService::new(
		ToastyUserRepository::new(db.clone()),
		ToastySessionRepository::new(db.clone()),
		Argon2PasswordHasher,
		tokens,
		mailer,
		config.security.refresh_token.expires_in_seconds,
	);

	let files = FilesService::new(
		ToastyFileRepository::new(db.clone()),
		LocalFileStorage::new(&config.files.upload_dir, &config.files.base_url),
	);

	Ok(Services {
		identity: Arc::new(identity),
		files: Arc::new(files),
	})
}

/// Assembles the HTTP router. The composition root decides which routers
/// sit behind the auth guard; the modules only provide their routes.
pub fn router(services: &Services, config: &AppConfig) -> Router {
	let auth_layer = from_fn_with_state(
		services.identity.clone(),
		identity_http::authenticate::<ProductionPorts>,
	);

	let auth_routes = identity_http::routes::auth_public_router(services.identity.clone())
		.merge(identity_http::routes::auth_protected_router(services.identity.clone()).layer(auth_layer.clone()));

	let users_routes = identity_http::routes::users_router(services.identity.clone()).layer(auth_layer.clone());

	let files_routes = files_http::files_router(services.files.clone()).layer(auth_layer);

	Router::new()
		.nest("/api/auth", auth_routes)
		.nest("/api/users", users_routes)
		.nest("/api/files", files_routes)
		.nest_service("/uploads", ServeDir::new(&config.files.upload_dir))
		.layer(from_fn(shared_infrastructure::i18n::middleware))
		.layer(CookieManagerLayer::new())
		.layer(TraceLayer::new_for_http())
}
