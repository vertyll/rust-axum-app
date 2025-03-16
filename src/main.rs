rust_i18n::i18n!("translations");

use crate::common::r#struct::app_state::AppState;
use crate::common::r#struct::token_state::TokenState;

use crate::auth::services::refresh_token_service::{RefreshTokenService, RefreshTokenServiceTrait};
use axum::middleware::from_fn_with_state;
use database::seeders;
use migration::{Migrator, MigratorTrait};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app_module;
mod auth;
mod common;
mod config;
mod database;
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
	let app_config = config::app_config::AppConfig::init().expect("Could not initialize the application configuration");

	// Database connection
	let db = database::connection::connect(&app_config.database)
		.await
		.expect("Could not connect to the database");

	// Run database migrations
	Migrator::up(&db, None).await.expect("Could not upgrade the database");

	// Run seeders
	seeders::run_seeders(&db).await.expect("Could not run seeders");

	// Create AppState
	let app_state = AppState::new(db.clone());

	// Create TokenState
	let token_state = TokenState::new(
		app_config.security.jwt_access_token_secret,
		app_config.security.jwt_access_token_expires_in,
		app_config.security.jwt_refresh_token_secret,
		app_config.security.jwt_refresh_token_expires_in,
	);

	// CRON jobs
	// Run a job to clean expired tokens every 24 hours
	let token_state_clone = token_state.clone();
	let app_state_clone = app_state.clone();
	tokio::spawn(async move {
		let refresh_token_service = Arc::new(RefreshTokenService::new(app_state_clone, token_state_clone));
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

	// App configuration
	let app = app_module::configure(app_state, token_state).await;

	let addr = SocketAddr::from(([127, 0, 0, 1], app_config.server.port));
	tracing::info!("Server is running on: http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.expect("Could not bind to the address");

	axum::serve(listener, app).await.expect("Server failed to start");
}
