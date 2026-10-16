//! Configuration, from environment variables by their exact names (see
//! `.env.example`). Topcoat itself reads `HOST` and `PORT`.

use std::env;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("unknown APP_ENVIRONMENT: {0} (use local|production)")]
    UnknownEnvironment(String),
    #[error("invalid value for {key}: {value}")]
    Invalid { key: &'static str, value: String },
    #[error("{0} must be set when APP_ENVIRONMENT is not local")]
    Missing(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Local,
    Production,
}

impl FromStr for Environment {
    type Err = ConfigError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "local" => Ok(Environment::Local),
            "production" => Ok(Environment::Production),
            other => Err(ConfigError::UnknownEnvironment(other.to_owned())),
        }
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Environment::Local => "local",
            Environment::Production => "production",
        })
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub environment: Environment,
    pub backend_url: String,
    pub backend_timeout: Duration,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let environment = parse("APP_ENVIRONMENT", Environment::Local)?;
        let backend_url = match (env::var("BACKEND_INTERNAL_URL"), environment) {
            (Ok(url), _) => url,
            (Err(_), Environment::Local) => "http://localhost:8080".to_owned(),
            (Err(_), Environment::Production) => return Err(ConfigError::Missing("BACKEND_INTERNAL_URL")),
        };
        Ok(Self {
            environment,
            backend_url,
            backend_timeout: Duration::from_secs(parse("BACKEND_TIMEOUT_SECONDS", 30)?),
        })
    }

    /// Cookies this application sets are `Secure` everywhere but on a local
    /// machine without TLS.
    #[must_use]
    pub fn secure_cookies(&self) -> bool {
        self.environment != Environment::Local
    }
}

fn parse<T: FromStr>(key: &'static str, default: T) -> Result<T, ConfigError> {
    match env::var(key) {
        Err(_) => Ok(default),
        Ok(value) => value.parse().map_err(|_| ConfigError::Invalid { key, value }),
    }
}
