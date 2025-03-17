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
	let app_dependencies = initialize_app_state(db.clone(), app_config_arc.clone());

	// CRON jobs
	// Run a job to clean expired tokens every 24 hours
	spawn_token_cleanup_job(app_dependencies.refresh_token_service.clone());

	let app = app_module::configure(app_config_arc.clone(), app_dependencies).await;

	let addr = SocketAddr::from(([127, 0, 0, 1], app_config.server.app_port));
	tracing::info!("Server is running on: http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.expect("Could not bind to the address");

	axum::serve(listener, app).await.expect("Server failed to start");
}

#[derive(Clone)]
struct AppState {
	users_service: Arc<dyn UsersServiceTrait>,
	auth_service: Arc<dyn AuthServiceTrait>,
	refresh_token_service: Arc<dyn RefreshTokenServiceTrait>,
	email_service: Arc<dyn EmailsServiceTrait>,
	user_roles_service: Arc<dyn UserRolesServiceTrait>,
	confirmation_token_service: Arc<dyn ConfirmationTokenServiceTrait>,
	files_service: Arc<dyn FilesServiceTrait>,
	roles_service: Arc<RolesService>,
	refresh_token_repository: Arc<RefreshTokenRepository>,
	users_repository: Arc<UsersRepository>,
	roles_repository: Arc<RolesRepository>,
	user_roles_repository: Arc<UserRolesRepository>,
}

// Dependency injection
fn initialize_app_state(db: DatabaseConnection, config: Arc<AppConfig>) -> AppState {
	// 1. Repositories first
	let refresh_token_repository = Arc::new(RefreshTokenRepository::new(db.clone()));
	let users_repository = Arc::new(UsersRepository::new(db.clone()));
	let roles_repository = Arc::new(RolesRepository::new(db.clone()));
	let user_roles_repository = Arc::new(UserRolesRepository::new(db.clone()));
	let files_repository = Arc::new(FilesRepository::new(db.clone()));

	// 2. Basic services with no dependencies on other services
	let email_service = Arc::new(EmailsService::new(config.clone()));
	let confirmation_token_service = Arc::new(ConfirmationTokenService::new(config.clone()));
	let roles_service = Arc::new(RolesService::new(roles_repository.clone()));
	let user_roles_service = Arc::new(UserRolesService::new(user_roles_repository.clone()));

	// 3. Services depending on other services
	let users_service = Arc::new(UsersService::new(
		users_repository.clone(),
		user_roles_service.clone(),
		email_service.clone(),
		confirmation_token_service.clone(),
		config.clone(),
	));

	let auth_service = Arc::new(AuthService::new(
		users_service.clone(),
		user_roles_service.clone(),
		config.clone(),
	));

	let refresh_token_service = Arc::new(RefreshTokenService::new(
		refresh_token_repository.clone(),
		user_roles_service.clone(),
		config.clone(),
	));

	let files_service = Arc::new(FilesService::new(files_repository.clone(), config.clone()));

	AppState {
		users_service,
		auth_service,
		refresh_token_service,
		email_service,
		user_roles_service,
		confirmation_token_service,
		files_service,
		roles_service,
		refresh_token_repository,
		users_repository,
		roles_repository,
		user_roles_repository,
	}
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
