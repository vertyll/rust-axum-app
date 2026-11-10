//! Application configuration, loaded from environment variables by their
//! exact names (see `.env.example`).

use std::env;
use std::fmt::Display;
use std::num::NonZeroU32;
use std::ops::RangeInclusive;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use secrecy::{ExposeSecret, SecretString};

use crate::files::domain::StorageKind;

/// The name of the session cookie where it is `Secure`: the `__Host-` prefix
/// makes the browser keep it to this host, over TLS, on every path.
const SECURE_SESSION_COOKIE: &str = "__Host-RUST_AXUM_APP_SESSION";

/// The name of the session cookie on a local machine without TLS, where the
/// `__Host-` prefix, which requires `Secure`, cannot be used.
const LOCAL_SESSION_COOKIE: &str = "RUST_AXUM_APP_SESSION";

const INACTIVITY_TIMEOUT_SECONDS: RangeInclusive<i64> = 60..=604_800;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub keycloak: KeycloakConfig,
    pub sessions: SessionsConfig,
    pub files: FilesConfig,
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
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: SecretString,
    pub name: String,
    pub max_connections: NonZeroU32,
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
    /// The Redis URL, which may hold a password.
    pub redis_url: SecretString,
    pub redis_key_prefix: String,
    /// Whether the session cookie is `Secure`: everywhere but in `local`.
    pub cookie_secure: bool,
    pub inactivity_timeout_seconds: i64,
}

impl SessionsConfig {
    #[must_use]
    pub fn cookie_name(&self) -> &'static str {
        if self.cookie_secure {
            SECURE_SESSION_COOKIE
        } else {
            LOCAL_SESSION_COOKIE
        }
    }
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

impl AppConfig {
    pub fn load() -> Result<Self> {
        Self::from_variables(&|key| env::var(key).ok())
    }

    fn from_variables(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self> {
        let vars = Variables(lookup);
        let environment: Environment = vars
            .get("APP_ENVIRONMENT")
            .context("APP_ENVIRONMENT must be set (use local|production)")?
            .parse()?;
        let config = Self {
            server: ServerConfig {
                host: vars.or("APP_HOST", "127.0.0.1"),
                port: vars.parse("APP_PORT", 8080)?,
                environment,
            },
            database: DatabaseConfig {
                host: vars.or("DB_HOST", "localhost"),
                port: vars.parse("DB_PORT", 5432)?,
                username: vars.or("DB_USERNAME", "postgres"),
                password: vars.or("DB_PASSWORD", "postgres").into(),
                name: vars.or("DB_NAME", "rust_axum_app"),
                max_connections: vars.parse("DB_MAX_CONNECTIONS", NonZeroU32::new(10).expect("ten is not zero"))?,
            },
            keycloak: KeycloakConfig {
                realm_url: vars.or("KEYCLOAK_REALM_URL", "http://localhost:9000/realms/rust-axum-app"),
                client_id: vars.or("KEYCLOAK_CLIENT_ID", "rust-axum-app"),
                client_secret: vars.or("KEYCLOAK_CLIENT_SECRET", "rust-axum-app-local-secret").into(),
                audience: vars.or("KEYCLOAK_AUDIENCE", "rust-axum-app"),
                callback_url: vars.or("AUTH_CALLBACK_URL", "http://localhost:3000/api/auth/callback"),
                post_login_url: vars.or("AUTH_POST_LOGIN_URL", "http://localhost:3000/"),
            },
            sessions: SessionsConfig {
                redis_url: vars.or("REDIS_URL", "redis://localhost:6379").into(),
                redis_key_prefix: vars.or("REDIS_KEY_PREFIX", "rust-axum-app"),
                cookie_secure: !environment.is_local(),
                inactivity_timeout_seconds: vars.parse_in(
                    "SESSION_INACTIVITY_TIMEOUT",
                    36_000,
                    INACTIVITY_TIMEOUT_SECONDS,
                )?,
            },
            files: FilesConfig {
                storage: vars.parse("FILES_STORAGE", StorageKind::S3)?,
                max_size: vars.parse("FILES_MAX_SIZE", 25 * 1024 * 1024)?,
                upload_dir: vars.or("FILES_UPLOAD_DIR", "uploads"),
                base_url: vars.or("FILES_BASE_URL", "/uploads"),
                signing_key: vars
                    .or("FILES_SIGNING_KEY", "rust-axum-app-local-files-signing-key")
                    .into(),
                s3: S3Config {
                    endpoint: vars.or("S3_ENDPOINT", "http://localhost:3900"),
                    public_endpoint: vars.or("S3_PUBLIC_ENDPOINT", &vars.or("S3_ENDPOINT", "http://localhost:3900")),
                    region: vars.or("S3_REGION", "garage"),
                    bucket: vars.or("S3_BUCKET", "rust-axum-app-files"),
                    access_key: vars.or("S3_ACCESS_KEY", "GK3a7c1e9b5d2f4086a1c3e5b7"),
                    secret_key: vars
                        .or(
                            "S3_SECRET_KEY",
                            "9d4f2b7e1a6c3058b9e2d7f4a1c6b3e8d5f2a9c7e4b1d8f3a6c9e2b5d7f1a4c8",
                        )
                        .into(),
                },
            },
        };

        config.validate(&vars)?;
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
            files_storage = %self.files.storage,
            uploads_dir = %self.files.upload_dir,
            "configuration loaded"
        );
    }

    /// Outside `local` every deployment-specific value has to come from the
    /// environment; a local default (localhost, placeholder secret) would
    /// otherwise start the service against the wrong database, Keycloak or
    /// Redis.
    fn validate(&self, vars: &Variables) -> Result<()> {
        if self.server.environment.is_local() {
            return Ok(());
        }

        let required = [
            "DB_HOST",
            "DB_USERNAME",
            "DB_PASSWORD",
            "DB_NAME",
            "KEYCLOAK_REALM_URL",
            "KEYCLOAK_CLIENT_SECRET",
            "AUTH_CALLBACK_URL",
            "AUTH_POST_LOGIN_URL",
            "REDIS_URL",
            "FILES_SIGNING_KEY",
        ];
        let object_store = ["S3_ENDPOINT", "S3_BUCKET", "S3_ACCESS_KEY", "S3_SECRET_KEY"];
        let uses_object_store = self.files.storage == StorageKind::S3;
        for key in required
            .into_iter()
            .chain(object_store.into_iter().filter(|_| uses_object_store))
        {
            if vars.get(key).is_none_or(|value| value.trim().is_empty()) {
                bail!("{key} must be set when APP_ENVIRONMENT is not local");
            }
        }

        Ok(())
    }
}

struct Variables<'a>(&'a dyn Fn(&str) -> Option<String>);

impl Variables<'_> {
    fn get(&self, key: &str) -> Option<String> {
        (self.0)(key)
    }

    fn or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }

    fn parse<T>(&self, key: &str, default: T) -> Result<T>
    where
        T: FromStr,
        T::Err: Display,
    {
        match self.get(key) {
            None => Ok(default),
            Some(raw) => raw
                .parse::<T>()
                .map_err(|err| anyhow::anyhow!("{err}"))
                .with_context(|| format!("invalid value for {key}")),
        }
    }

    fn parse_in<T>(&self, key: &str, default: T, range: RangeInclusive<T>) -> Result<T>
    where
        T: FromStr + PartialOrd + Display,
        T::Err: Display,
    {
        let value = self.parse(key, default)?;
        if !range.contains(&value) {
            bail!(
                "invalid value for {key}: {value} (use {} to {})",
                range.start(),
                range.end()
            );
        }
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{AppConfig, Environment};

    fn load(variables: &[(&str, &str)]) -> anyhow::Result<AppConfig> {
        let variables: HashMap<String, String> = variables
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        AppConfig::from_variables(&|key| variables.get(key).cloned())
    }

    fn production() -> Vec<(&'static str, &'static str)> {
        vec![
            ("APP_ENVIRONMENT", "production"),
            ("DB_HOST", "db.internal"),
            ("DB_USERNAME", "app"),
            ("DB_PASSWORD", "database-secret"),
            ("DB_NAME", "app"),
            ("KEYCLOAK_REALM_URL", "https://auth.example.com/realms/app"),
            ("KEYCLOAK_CLIENT_SECRET", "keycloak-secret"),
            ("AUTH_CALLBACK_URL", "https://app.example.com/api/auth/callback"),
            ("AUTH_POST_LOGIN_URL", "https://app.example.com/"),
            ("REDIS_URL", "redis://:redis-secret@redis.internal:6379"),
            ("FILES_SIGNING_KEY", "signing-secret"),
            ("S3_ENDPOINT", "https://s3.example.com"),
            ("S3_BUCKET", "files"),
            ("S3_ACCESS_KEY", "access"),
            ("S3_SECRET_KEY", "storage-secret"),
        ]
    }

    #[test]
    fn the_debug_output_hides_every_secret() {
        let config = load(&production()).unwrap();

        let printed = format!("{config:?}");

        for secret in [
            "database-secret",
            "keycloak-secret",
            "redis-secret",
            "signing-secret",
            "storage-secret",
        ] {
            assert!(!printed.contains(secret), "{secret} is printed");
        }
    }

    #[test]
    fn a_missing_environment_is_refused() {
        let refused = load(&[]).unwrap_err();

        assert!(refused.to_string().contains("APP_ENVIRONMENT"), "{refused}");
    }

    #[test]
    fn local_falls_back_to_the_containers_and_a_plain_cookie() {
        let config = load(&[("APP_ENVIRONMENT", "local")]).unwrap();

        assert_eq!(config.server.environment, Environment::Local);
        assert!(!config.sessions.cookie_secure);
        assert_eq!(config.sessions.cookie_name(), "RUST_AXUM_APP_SESSION");
    }

    #[test]
    fn production_makes_the_session_cookie_secure_and_host_only() {
        let config = load(&production()).unwrap();

        assert!(config.sessions.cookie_secure);
        assert_eq!(config.sessions.cookie_name(), "__Host-RUST_AXUM_APP_SESSION");
    }

    #[test]
    fn production_refuses_a_missing_setting() {
        let variables: Vec<_> = production()
            .into_iter()
            .filter(|(key, _)| *key != "REDIS_URL")
            .collect();

        let refused = load(&variables).unwrap_err();

        assert!(refused.to_string().contains("REDIS_URL"), "{refused}");
    }

    #[test]
    fn values_out_of_range_are_refused() {
        for (key, value) in [
            ("SESSION_INACTIVITY_TIMEOUT", "0"),
            ("SESSION_INACTIVITY_TIMEOUT", "-1"),
            ("SESSION_INACTIVITY_TIMEOUT", "31536000"),
            ("DB_MAX_CONNECTIONS", "0"),
        ] {
            let refused = load(&[("APP_ENVIRONMENT", "local"), (key, value)]).unwrap_err();

            assert!(refused.to_string().contains(key), "{key}={value}: {refused}");
        }
    }
}
