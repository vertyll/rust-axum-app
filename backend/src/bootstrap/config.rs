//! Application configuration, loaded from environment variables by their
//! exact names (see `.env.example`).

use std::env;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use secrecy::{ExposeSecret, SecretString};

use crate::files::StorageKind;

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
    #[must_use]
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
    pub password: SecretString,
    pub name: String,
    pub max_connections: u32,
}

impl DatabaseConfig {
    /// The connection URL, which holds the password.
    #[must_use]
    pub fn url(&self) -> SecretString {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.name
        )
        .into()
    }
}

#[derive(Debug, Clone)]
pub struct KeycloakConfig {
    pub realm_url: String,
    pub client_id: String,
    pub client_secret: SecretString,
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
    /// The storage new uploads go to.
    pub storage: StorageKind,
    /// The largest upload allowed, in bytes.
    pub max_size: u64,
    pub upload_dir: String,
    pub base_url: String,
    /// Signs the links to the local disk.
    pub signing_key: SecretString,
    pub s3: S3Config,
}

#[derive(Debug, Clone)]
pub struct S3Config {
    pub endpoint: String,
    pub public_endpoint: String,
    pub region: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: SecretString,
}

#[derive(Debug, Clone)]
pub struct EmailsConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: SecretString,
    pub from: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let config = Self {
            server: ServerConfig {
                host: env_or("APP_HOST", "127.0.0.1"),
                port: env_parse("APP_PORT", 8080)?,
                environment: env_parse("APP_ENVIRONMENT", Environment::Local)?,
                url: env_or("APP_URL", "http://localhost:8080"),
            },
            database: DatabaseConfig {
                host: env_or("DB_HOST", "localhost"),
                port: env_parse("DB_PORT", 5432)?,
                username: env_or("DB_USERNAME", "postgres"),
                password: env_or("DB_PASSWORD", "postgres").into(),
                name: env_or("DB_NAME", "rust_axum_app"),
                max_connections: env_parse("DB_MAX_CONNECTIONS", 10)?,
            },
            keycloak: KeycloakConfig {
                realm_url: env_or("KEYCLOAK_REALM_URL", "http://localhost:9000/realms/rust-axum-app"),
                client_id: env_or("KEYCLOAK_CLIENT_ID", "rust-axum-app"),
                client_secret: env_or("KEYCLOAK_CLIENT_SECRET", "rust-axum-app-local-secret").into(),
                audience: env_or("KEYCLOAK_AUDIENCE", "rust-axum-app"),
                callback_url: env_or("AUTH_CALLBACK_URL", "http://localhost:3000/api/auth/callback"),
                post_login_url: env_or("AUTH_POST_LOGIN_URL", "http://localhost:3000/"),
            },
            sessions: SessionsConfig {
                redis_url: env_or("REDIS_URL", "redis://localhost:6379"),
                redis_key_prefix: env_or("REDIS_KEY_PREFIX", "rust-axum-app"),
                cookie_secure: env_parse("SESSION_COOKIE_SECURE", false)?,
                inactivity_timeout_seconds: env_parse("SESSION_INACTIVITY_TIMEOUT", 36_000)?,
            },
            files: FilesConfig {
                storage: env_parse("FILES_STORAGE", StorageKind::S3)?,
                max_size: env_parse("FILES_MAX_SIZE", 25 * 1024 * 1024)?,
                upload_dir: env_or("FILES_UPLOAD_DIR", "uploads"),
                base_url: env_or("FILES_BASE_URL", "/uploads"),
                signing_key: env_or("FILES_SIGNING_KEY", "rust-axum-app-local-files-signing-key").into(),
                s3: S3Config {
                    endpoint: env_or("S3_ENDPOINT", "http://localhost:3900"),
                    public_endpoint: env_or("S3_PUBLIC_ENDPOINT", &env_or("S3_ENDPOINT", "http://localhost:3900")),
                    region: env_or("S3_REGION", "garage"),
                    bucket: env_or("S3_BUCKET", "rust-axum-app-files"),
                    access_key: env_or("S3_ACCESS_KEY", "GK3a7c1e9b5d2f4086a1c3e5b7"),
                    secret_key: env_or(
                        "S3_SECRET_KEY",
                        "9d4f2b7e1a6c3058b9e2d7f4a1c6b3e8d5f2a9c7e4b1d8f3a6c9e2b5d7f1a4c8",
                    )
                    .into(),
                },
            },
            emails: EmailsConfig {
                smtp_host: env_or("SMTP_HOST", "localhost"),
                smtp_port: env_parse("SMTP_PORT", 1025)?,
                smtp_username: env_or("SMTP_USERNAME", ""),
                smtp_password: env_or("SMTP_PASSWORD", "").into(),
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
            files_storage = %self.files.storage,
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
            "FILES_SIGNING_KEY",
        ];
        let object_store = ["S3_ENDPOINT", "S3_BUCKET", "S3_ACCESS_KEY", "S3_SECRET_KEY"];
        let uses_object_store = self.files.storage == StorageKind::S3;
        for key in required
            .into_iter()
            .chain(object_store.into_iter().filter(|_| uses_object_store))
        {
            if env::var(key).map_or(true, |value| value.trim().is_empty()) {
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

#[cfg(test)]
mod tests {
    use super::AppConfig;

    #[test]
    fn the_debug_output_hides_every_secret() {
        let mut config = AppConfig::load().unwrap();
        config.database.password = "database-secret".to_owned().into();
        config.keycloak.client_secret = "keycloak-secret".to_owned().into();
        config.emails.smtp_password = "smtp-secret".to_owned().into();
        config.files.signing_key = "signing-secret".to_owned().into();
        config.files.s3.secret_key = "storage-secret".to_owned().into();

        let printed = format!("{config:?}");

        for secret in [
            "database-secret",
            "keycloak-secret",
            "smtp-secret",
            "signing-secret",
            "storage-secret",
        ] {
            assert!(!printed.contains(secret), "{secret} is printed");
        }
    }
}
