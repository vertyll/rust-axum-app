rust_i18n::i18n!("translations");

use crate::common::r#struct::app_state::AppState;
use axum::middleware::from_fn_with_state;
use migration::{Migrator, MigratorTrait};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app_module;
mod auth;
mod common;
mod config;
mod database;
mod i18n;
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
	Migrator::up(&db, None).await.unwrap();

	// Create AppState
	let app_state = AppState::new(db.clone(), app_config.security.jwt_access_token_secret);

	// App configuration
	let app = app_module::configure(app_state, app_config.security.jwt_access_token_expires_in).await;

	let addr = SocketAddr::from(([127, 0, 0, 1], app_config.server.port));
	tracing::info!("Server is running on: http://{}", addr);

	let listener = tokio::net::TcpListener::bind(addr)
		.await
		.expect("Could not bind to the address");

	axum::serve(listener, app).await.expect("Server failed to start");
}
