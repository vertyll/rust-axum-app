use crate::auth::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::auth::services::auth_service::{AuthService, AuthServiceTrait};
use crate::auth::services::confirmation_token_service::{ConfirmationTokenService, ConfirmationTokenServiceTrait};
use crate::auth::services::refresh_token_service::{RefreshTokenService, RefreshTokenServiceTrait};
use crate::config::app_config::AppConfig;
use crate::emails::services::emails_service::{EmailsService, EmailsServiceTrait};
use crate::files::repositories::files_repository::FilesRepository;
use crate::files::services::files_service::{FilesService, FilesServiceTrait};
use crate::roles::repositories::roles_repository::RolesRepository;
use crate::roles::repositories::user_roles_repository::UserRolesRepository;
use crate::roles::services::roles_service::RolesService;
use crate::roles::services::user_roles_service::{UserRolesService, UserRolesServiceTrait};
use crate::users::repositories::users_repository::UsersRepository;
use crate::users::services::users_service::{UsersService, UsersServiceTrait};
use axum::Extension;
use database::seeders;
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use crate::di::{AppConfigImpl, AppConfigTrait, DatabaseConnectionImpl, DatabaseConnectionTrait};
use crate::di::module;

rust_i18n::i18n!("translations");

mod app_module;
mod auth;
mod common;
mod config;
mod database;
mod emails;
mod files;
mod i18n;
mod roles;
mod users;
mod di;

#[tokio::main]
async fn main() {
	dotenv::dotenv().ok();

	// Initialize logger
	tracing_subscriber::registry()
		.with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	// App configuration
	let app_config = AppConfig::init().expect("Could not initialize the application configuration");
	let app_config_arc = Arc::new(app_config.clone());

	// Database connection
	let db = database::connection::connect(&app_config.database)
		.await
		.expect("Could not connect to the database");

	// Run database migrations
	Migrator::up(&db, None).await.expect("Could not upgrade the database");

	// Run seeders
	seeders::run_seeders(&db).await.expect("Could not run seeders");

	// Initialize services
	let di_module = Arc::new(module::initialize_di(db.clone(), app_config_arc.clone()));

	// CRON jobs
	// Run a job to clean expired tokens every 24 hours
	spawn_token_cleanup_job(di_module.refresh_token_service.clone());

	let app = app_module::configure(app_config_arc.clone(), di_module).await;

	let addr = SocketAddr::from(([127, 0, 0, 1], app_config.server.app_port));
	tracing::info!("Server is running on: http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.expect("Could not bind to the address");

	axum::serve(listener, app).await.expect("Server failed to start");
}

// CRON jobs
fn spawn_token_cleanup_job(refresh_token_service: Arc<dyn RefreshTokenServiceTrait>) {
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(86400)); // 24 hours
		loop {
			interval.tick().await;
			if let Err(err) = refresh_token_service.clean_expired_tokens().await {
				tracing::error!("Error cleaning expired tokens: {:?}", err);
			} else {
				tracing::info!("Successfully cleaned expired tokens");
			}
		}
	});
}
