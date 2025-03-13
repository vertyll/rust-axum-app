use crate::config::database_config::DatabaseConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
	pub port: u16,
	pub host: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
	pub jwt_access_token_secret: String,
	pub jwt_access_token_expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub security: SecurityConfig,
}

impl AppConfig {
	pub fn init() -> Result<Self, config::ConfigError> {
		let config = config::Config::builder()
			// Server
			.set_default("server.port", 3000)?
			.set_default("server.host", "127.0.0.1")?
			// Database
			.set_default("database.host", "localhost")?
			.set_default("database.port", 5432)?
			.set_default("database.username", "postgres")?
			.set_default("database.password", "postgres")?
			.set_default("database.db_name", "rust_axum_app")?
			.set_default("database.max_connections", 5)?
			.set_default("database.min_connections", 1)?
			// Security
			.set_default("security.jwt_access_token_secret", "default_secret")?
			.set_default("security.jwt_access_token_expires_in", "60000")?
			// Config file (optional)
			.add_source(config::File::with_name("config").required(false))
			// Environment variables
			.add_source(config::Environment::with_prefix("APP").separator("_"))
			.add_source(config::Environment::with_prefix("DB").separator("_"))
			.add_source(config::Environment::with_prefix("JWT").separator("_"))
			.build()?;

		let app_config: AppConfig = config.try_deserialize()?;

		Ok(app_config)
	}
}
