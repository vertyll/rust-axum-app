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
	pub jwt_refresh_token_secret: String,
	pub jwt_refresh_token_expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilesConfig {
	pub upload_dir: String,
	pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub security: SecurityConfig,
	pub files: FilesConfig,
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
			.set_default("security.jwt_refresh_token_secret", "default_secret")?
			.set_default("security.jwt_refresh_token_expires_in", "86400000")?
			// Files
			.set_default("files.upload_dir", "uploads")?
			.set_default("files.base_url", "/uploads")?
			// Config file (optional)
			.add_source(config::File::with_name("config").required(false))
			// Environment variables
			.add_source(config::Environment::with_prefix("APP").separator("_"))
			.add_source(config::Environment::with_prefix("DB").separator("_"))
			.add_source(config::Environment::with_prefix("JWT").separator("_"))
			.add_source(config::Environment::with_prefix("FILES").separator("_"))
			.build()?;

		let app_config: AppConfig = config.try_deserialize()?;

		Ok(app_config)
	}
}
