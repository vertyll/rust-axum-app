//! Configuration, from environment variables by their exact names (see
//! `.env.example`). Topcoat itself reads `HOST` and `PORT`.

use std::env;
use std::fmt;
use std::num::NonZeroU64;
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
    #[error("APP_ENVIRONMENT must be set (use local|production)")]
    NoEnvironment,
}

/// The back-end's session cookie where it is `Secure`, with the `__Host-`
/// prefix; the back-end names it so in the same environments.
const SECURE_SESSION_COOKIE: &str = "__Host-RUST_AXUM_APP_SESSION";

/// The back-end's session cookie on a local machine without TLS.
const LOCAL_SESSION_COOKIE: &str = "RUST_AXUM_APP_SESSION";

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
    pub storage_url: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        Self::from_variables(&|key| env::var(key).ok())
    }

    fn from_variables(lookup: &dyn Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |key: &str| lookup(key).filter(|value| !value.trim().is_empty());
        let environment: Environment = get("APP_ENVIRONMENT").ok_or(ConfigError::NoEnvironment)?.parse()?;
        let backend_url = match (get("BACKEND_INTERNAL_URL"), environment) {
            (Some(url), _) => url,
            (None, Environment::Local) => "http://localhost:8080".to_owned(),
            (None, Environment::Production) => return Err(ConfigError::Missing("BACKEND_INTERNAL_URL")),
        };
        let backend_timeout = match get("BACKEND_TIMEOUT_SECONDS") {
            None => 30,
            Some(value) => value
                .parse::<NonZeroU64>()
                .map_err(|_| ConfigError::Invalid {
                    key: "BACKEND_TIMEOUT_SECONDS",
                    value,
                })?
                .get(),
        };
        let storage_url = match (get("S3_PUBLIC_ENDPOINT"), environment) {
            (Some(url), _) if is_origin(&url) => url,
            (Some(url), _) => {
                return Err(ConfigError::Invalid {
                    key: "S3_PUBLIC_ENDPOINT",
                    value: url,
                });
            }
            (None, Environment::Local) => "http://localhost:3900".to_owned(),
            (None, Environment::Production) => return Err(ConfigError::Missing("S3_PUBLIC_ENDPOINT")),
        };
        Ok(Self {
            environment,
            backend_url,
            backend_timeout: Duration::from_secs(backend_timeout),
            storage_url,
        })
    }

    /// Cookies this application sets are `Secure` everywhere but on a local
    /// machine without TLS.
    #[must_use]
    pub fn secure_cookies(&self) -> bool {
        self.environment != Environment::Local
    }

    /// The name of the back-end's session cookie, which this server passes on.
    #[must_use]
    pub fn session_cookie(&self) -> &'static str {
        if self.secure_cookies() {
            SECURE_SESSION_COOKIE
        } else {
            LOCAL_SESSION_COOKIE
        }
    }
}

/// Whether a value is an `http` or `https` origin that a policy can name as it
/// is, with nothing that would end its directive or start another.
fn is_origin(value: &str) -> bool {
    let rest = value.strip_prefix("https://").or_else(|| value.strip_prefix("http://"));
    rest.is_some_and(|host| {
        !host.is_empty() && !host.contains(['/', ';', ',', '\'', '"']) && !host.chars().any(char::is_whitespace)
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use super::{AppConfig, ConfigError};

    fn load(variables: &[(&str, &str)]) -> Result<AppConfig, ConfigError> {
        let variables: HashMap<String, String> = variables
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
            .collect();
        AppConfig::from_variables(&|key| variables.get(key).cloned())
    }

    #[test]
    fn a_missing_environment_is_refused() {
        assert!(matches!(load(&[]), Err(ConfigError::NoEnvironment)));
    }

    #[test]
    fn local_reaches_the_local_back_end_with_a_plain_session_cookie() {
        let config = load(&[("APP_ENVIRONMENT", "local")]).unwrap();

        assert_eq!(config.backend_url, "http://localhost:8080");
        assert_eq!(config.backend_timeout, Duration::from_secs(30));
        assert_eq!(config.session_cookie(), "RUST_AXUM_APP_SESSION");
        assert_eq!(config.storage_url, "http://localhost:3900");
    }

    #[test]
    fn production_requires_the_storage_the_browser_uploads_to() {
        assert!(matches!(
            load(&[
                ("APP_ENVIRONMENT", "production"),
                ("BACKEND_INTERNAL_URL", "http://backend:8080"),
            ]),
            Err(ConfigError::Missing("S3_PUBLIC_ENDPOINT"))
        ));
    }

    #[test]
    fn a_storage_address_that_is_not_an_origin_is_refused() {
        for value in [
            "files.example.com",
            "https://",
            "https://files.example.com/bucket",
            "https://files.example.com; script-src *",
            "https://files.example.com https://elsewhere.example.com",
        ] {
            assert!(matches!(
                load(&[("APP_ENVIRONMENT", "local"), ("S3_PUBLIC_ENDPOINT", value)]),
                Err(ConfigError::Invalid {
                    key: "S3_PUBLIC_ENDPOINT",
                    ..
                })
            ));
        }
    }

    #[test]
    fn production_requires_the_back_end_and_names_a_host_only_cookie() {
        for url in [None, Some(""), Some("  ")] {
            let mut variables = vec![("APP_ENVIRONMENT", "production")];
            variables.extend(url.map(|url| ("BACKEND_INTERNAL_URL", url)));

            assert!(matches!(
                load(&variables),
                Err(ConfigError::Missing("BACKEND_INTERNAL_URL"))
            ));
        }

        let config = load(&[
            ("APP_ENVIRONMENT", "production"),
            ("BACKEND_INTERNAL_URL", "http://backend:8080"),
            ("S3_PUBLIC_ENDPOINT", "https://files.example.com"),
        ])
        .unwrap();
        assert!(config.secure_cookies());
        assert_eq!(config.session_cookie(), "__Host-RUST_AXUM_APP_SESSION");
    }

    #[test]
    fn a_timeout_of_zero_is_refused() {
        for value in ["0", "-1", "soon"] {
            assert!(matches!(
                load(&[("APP_ENVIRONMENT", "local"), ("BACKEND_TIMEOUT_SECONDS", value)]),
                Err(ConfigError::Invalid {
                    key: "BACKEND_TIMEOUT_SECONDS",
                    ..
                })
            ));
        }
    }
}
