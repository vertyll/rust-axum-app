//! Application configuration, loaded from environment variables by their
//! exact names (see `.env.example`). Outside development, startup fails
//! when the token secrets are unset instead of signing with placeholders.

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct AppConfig {
	pub server: ServerConfig,
	pub database: DatabaseConfig,
	pub security: SecurityConfig,
	pub files: FilesConfig,
	pub emails: EmailsConfig,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
	Development,
	Production,
}

impl Environment {
	pub fn is_development(self) -> bool {
		self == Environment::Development
	}
}

impl FromStr for Environment {
	type Err = anyhow::Error;

	fn from_str(value: &str) -> Result<Self> {
		match value.to_ascii_lowercase().as_str() {
			"development" | "dev" => Ok(Environment::Development),
			"production" | "prod" => Ok(Environment::Production),
			other => bail!("unknown APP_ENVIRONMENT: {other} (use development|production)"),
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
pub struct TokenConfig {
	pub secret: String,
	pub expires_in_seconds: i64,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
	pub access_token: TokenConfig,
	pub refresh_token: TokenConfig,
	pub confirmation_token: TokenConfig,
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
	pub templates_dir: String,
}

impl AppConfig {
	pub fn load() -> Result<Self> {
		let config = Self {
			server: ServerConfig {
				host: env_or("APP_HOST", "127.0.0.1"),
				port: env_parse("APP_PORT", 3000)?,
				environment: env_parse("APP_ENVIRONMENT", Environment::Development)?,
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
			security: SecurityConfig {
				access_token: TokenConfig {
					secret: env_or("JWT_ACCESS_TOKEN_SECRET", "secret"),
					expires_in_seconds: env_parse("JWT_ACCESS_TOKEN_EXPIRES_IN", 3_600)?,
				},
				refresh_token: TokenConfig {
					secret: env_or("JWT_REFRESH_TOKEN_SECRET", "secret"),
					expires_in_seconds: env_parse("JWT_REFRESH_TOKEN_EXPIRES_IN", 2_592_000)?,
				},
				confirmation_token: TokenConfig {
					secret: env_or("CONFIRMATION_TOKEN_SECRET", "secret"),
					expires_in_seconds: env_parse("CONFIRMATION_TOKEN_EXPIRES_IN", 86_400)?,
				},
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
				from: env_or("EMAIL_FROM", "app@example.com"),
				templates_dir: env_or("EMAIL_TEMPLATES_DIR", "resources/templates/emails"),
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
			smtp_host = %self.emails.smtp_host,
			uploads_dir = %self.files.upload_dir,
			"configuration loaded"
		);
	}

	/// Production must not run on placeholder secrets — fail at boot instead.
	fn validate(&self) -> Result<()> {
		if self.server.environment.is_development() {
			return Ok(());
		}

		let required = [
			"JWT_ACCESS_TOKEN_SECRET",
			"JWT_REFRESH_TOKEN_SECRET",
			"CONFIRMATION_TOKEN_SECRET",
		];
		for key in required {
			if env::var(key).map(|value| value.trim().is_empty()).unwrap_or(true) {
				bail!("{key} must be set when APP_ENVIRONMENT is not development");
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
