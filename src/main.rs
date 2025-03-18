use crate::auth::services::refresh_token_service::IRefreshTokenService;
use crate::config::app_config::AppConfig;
use crate::di::module;
use axum::{Extension, Router};
use database::seeders;
use migration::{Migrator, MigratorTrait};
use sea_orm::DatabaseConnection;
use shaku::{HasComponent, HasProvider};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

rust_i18n::i18n!("translations");

mod app_module;
mod auth;
mod common;
mod config;
mod database;
mod di;
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
	let db_arc = Arc::new(db.clone());

	// Run database migrations
	Migrator::up(&db, None).await.expect("Could not upgrade the database");

	// Run seeders
	seeders::run_seeders(&db).await.expect("Could not run seeders");

	// Initialize DI module
	let di_module = module::initialize_di(db_arc, app_config_arc.clone());

	// Refresh token service for the cleanup job
	let refresh_token_service: Arc<dyn IRefreshTokenService> = di_module.resolve();

	// CRON jobs
	// Run a job to clean expired tokens every 24 hours
	spawn_token_cleanup_job(refresh_token_service);

	// Build application
	let app = app_module::configure(app_config_arc.clone(), di_module).await;

	let addr = SocketAddr::from(([127, 0, 0, 1], app_config.server.app_port));
	tracing::info!("Server is running on: http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.expect("Could not bind to the address");

	axum::serve(listener, app).await.expect("Server failed to start");
}

// CRON jobs
fn spawn_token_cleanup_job(refresh_token_service: Arc<dyn IRefreshTokenService>) {
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
