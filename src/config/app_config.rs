use crate::config::database_config::DatabaseConfig;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
	pub port: u16,
	pub host: String,
	pub environment: String,
	pub app_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtAccessTokenConfig {
	pub secret: String,
	pub expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JwtRefreshTokenConfig {
	pub secret: String,
	pub expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConfirmationTokenConfig {
	pub secret: String,
	pub expires_in: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TokensConfig {
	pub jwt_access_token: JwtAccessTokenConfig,
	pub jwt_refresh_token: JwtRefreshTokenConfig,
	pub confirmation_token: ConfirmationTokenConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SecurityConfig {
	pub tokens: TokensConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FilesConfig {
	pub upload_dir: String,
	pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EmailsConfig {
	pub smtp_host: String,
	pub smtp_port: u16,
	pub smtp_username: String,
	pub smtp_password: String,
	pub email_from: String,
	pub email_templates_dir: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub security: SecurityConfig,
	pub files: FilesConfig,
	pub emails: EmailsConfig,
}

impl AppConfig {
	pub fn init() -> Result<Self, config::ConfigError> {
		let config = config::Config::builder()
			// Server
			.set_default("server.port", 3000)?
			.set_default("server.host", "127.0.0.1")?
			.set_default("server.environment", "development")?
			.set_default("server.app_url", "http://localhost:3000")?
			// Database
			.set_default("database.host", "localhost")?
			.set_default("database.port", 5432)?
			.set_default("database.username", "postgres")?
			.set_default("database.password", "postgres")?
			.set_default("database.db_name", "rust_axum_app")?
			.set_default("database.max_connections", 5)?
			.set_default("database.min_connections", 1)?
			// Security
			.set_default("security.tokens.jwt_access_token.secret", "secret")?
			.set_default("security.tokens.jwt_access_token.expires_in", 3600)?
			.set_default("security.tokens.jwt_refresh_token.secret", "secret")?
			.set_default("security.tokens.jwt_refresh_token.expires_in", 2592000)?
			.set_default("security.tokens.confirmation_token.secret", "secret")?
			.set_default("security.tokens.confirmation_token.expires_in", 86400)?
			// Files
			.set_default("files.upload_dir", "uploads")?
			.set_default("files.base_url", "/uploads")?
			// Email
			.set_default("emails.smtp_host", "localhost")?
			.set_default("emails.smtp_port", 1025)?
			.set_default("emails.smtp_username", "")?
			.set_default("emails.smtp_password", "")?
			.set_default("emails.email_from", "app@example.com")?
			.set_default("emails.email_templates_dir", "resources/templates/emails")?
			// Config file (optional)
			.add_source(config::File::with_name("config").required(false))
			// Environment variables
			.add_source(config::Environment::with_prefix("APP").separator("_"))
			.add_source(config::Environment::with_prefix("DB").separator("_"))
			.add_source(config::Environment::with_prefix("JWT").separator("_"))
			.add_source(config::Environment::with_prefix("FILES").separator("_"))
			.add_source(config::Environment::with_prefix("EMAILS").separator("_"))
			.build()?;

		let app_config: AppConfig = config.try_deserialize()?;

		Ok(app_config)
	}
}
