//! Application configuration, loaded from environment variables by their
//! exact names (see `.env.example`). Outside local, startup fails when a
//! deployment-specific value is unset instead of falling back to a local one.

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub keycloak: KeycloakConfig,
	pub sessions: SessionsConfig,
	pub files: FilesConfig,
	pub emails: EmailsConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
	Local,
	Production,
}

impl Environment {
	pub fn is_local(self) -> bool {
		self == Environment::Local
	}
}

impl FromStr for Environment {
	type Err = anyhow::Error;

	fn from_str(value: &str) -> Result<Self> {
		match value.to_ascii_lowercase().as_str() {
			"local" => Ok(Environment::Local),
			"production" => Ok(Environment::Production),
			other => bail!("unknown APP_ENVIRONMENT: {other} (use local|production)"),
		}
	}
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
	pub host: String,
	pub port: u16,
	pub environment: Environment,
	pub url: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
	pub host: String,
	pub port: u16,
	pub username: String,
	pub password: String,
	pub name: String,
	pub max_connections: u32,
}

impl DatabaseConfig {
	pub fn url(&self) -> String {
		format!(
			"postgresql://{}:{}@{}:{}/{}",
			self.username, self.password, self.host, self.port, self.name
		)
	}
}

#[derive(Debug, Clone)]
pub struct KeycloakConfig {
	pub realm_url: String,
	pub client_id: String,
	pub client_secret: String,
	pub audience: String,
	pub callback_url: String,
	pub post_login_url: String,
}

#[derive(Debug, Clone)]
pub struct SessionsConfig {
	pub redis_url: String,
	pub redis_key_prefix: String,
	pub cookie_secure: bool,
	pub inactivity_timeout_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct FilesConfig {
	pub upload_dir: String,
	pub base_url: String,
}

#[derive(Debug, Clone)]
pub struct EmailsConfig {
	pub smtp_host: String,
	pub smtp_port: u16,
	pub smtp_username: String,
	pub smtp_password: String,
	pub from: String,
}

impl AppConfig {
	pub fn load() -> Result<Self> {
		let config = Self {
			server: ServerConfig {
				host: env_or("APP_HOST", "127.0.0.1"),
				port: env_parse("APP_PORT", 3000)?,
				environment: env_parse("APP_ENVIRONMENT", Environment::Local)?,
				url: env_or("APP_URL", "http://localhost:3000"),
			},
			database: DatabaseConfig {
				host: env_or("DB_HOST", "localhost"),
				port: env_parse("DB_PORT", 5432)?,
				username: env_or("DB_USERNAME", "postgres"),
				password: env_or("DB_PASSWORD", "postgres"),
				name: env_or("DB_NAME", "rust_axum_app"),
				max_connections: env_parse("DB_MAX_CONNECTIONS", 10)?,
			},
			keycloak: KeycloakConfig {
				realm_url: env_or("KEYCLOAK_REALM_URL", "http://localhost:9000/realms/rust-axum-app"),
				client_id: env_or("KEYCLOAK_CLIENT_ID", "rust-axum-app"),
				client_secret: env_or("KEYCLOAK_CLIENT_SECRET", "rust-axum-app-local-secret"),
				audience: env_or("KEYCLOAK_AUDIENCE", "rust-axum-app"),
				callback_url: env_or("AUTH_CALLBACK_URL", "http://localhost:3000/api/auth/callback"),
				post_login_url: env_or("AUTH_POST_LOGIN_URL", "http://localhost:3000/api/auth/session"),
			},
			sessions: SessionsConfig {
				redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
				redis_key_prefix: env_or("REDIS_KEY_PREFIX", "rust-axum-app"),
				cookie_secure: env_parse("SESSION_COOKIE_SECURE", false)?,
				inactivity_timeout_seconds: env_parse("SESSION_INACTIVITY_TIMEOUT", 36_000)?,
			},
			files: FilesConfig {
				upload_dir: env_or("FILES_UPLOAD_DIR", "uploads"),
				base_url: env_or("FILES_BASE_URL", "/uploads"),
			},
			emails: EmailsConfig {
				smtp_host: env_or("SMTP_HOST", "localhost"),
				smtp_port: env_parse("SMTP_PORT", 1025)?,
				smtp_username: env_or("SMTP_USERNAME", ""),
				smtp_password: env_or("SMTP_PASSWORD", ""),
				from: env_or("EMAIL_FROM", "no-reply@rust-axum-app.local"),
			},
		};

		config.validate()?;
		Ok(config)
	}

	/// One structured line ops can grep at boot; secrets are never logged.
	pub fn log_summary(&self) {
		tracing::info!(
			environment = ?self.server.environment,
			host = %self.server.host,
			port = self.server.port,
			database = %format_args!("{}:{}/{}", self.database.host, self.database.port, self.database.name),
			keycloak = %self.keycloak.realm_url,
			smtp_host = %self.emails.smtp_host,
			uploads_dir = %self.files.upload_dir,
			"configuration loaded"
		);
	}

	/// Outside `local` every deployment-specific value has to come from the
	/// environment; a local default (localhost, placeholder secret) would
	/// otherwise start the service against the wrong database, Keycloak, mail
	/// server or public URL.
	fn validate(&self) -> Result<()> {
		if self.server.environment.is_local() {
			return Ok(());
		}

		let required = [
			"APP_URL",
			"DB_HOST",
			"DB_USERNAME",
			"DB_PASSWORD",
			"DB_NAME",
			"SMTP_HOST",
			"EMAIL_FROM",
			"KEYCLOAK_REALM_URL",
			"KEYCLOAK_CLIENT_SECRET",
			"AUTH_CALLBACK_URL",
			"AUTH_POST_LOGIN_URL",
			"REDIS_URL",
			"SESSION_COOKIE_SECURE",
		];
		for key in required {
			if env::var(key).map(|value| value.trim().is_empty()).unwrap_or(true) {
				bail!("{key} must be set when APP_ENVIRONMENT is not local");
			}
		}

		Ok(())
	}
}

fn env_or(key: &str, default: &str) -> String {
	env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_parse<T>(key: &str, default: T) -> Result<T>
where
	T: FromStr,
	T::Err: Display,
{
	match env::var(key) {
		Err(_) => Ok(default),
		Ok(raw) => raw
			.parse::<T>()
			.map_err(|err| anyhow::anyhow!("{err}"))
			.with_context(|| format!("invalid value for {key}")),
	}
}
