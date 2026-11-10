//! Composition root — the only place naming concrete adapters. Dependency
//! injection is constructor calls plus the aliases below, all resolved at
//! compile time; nothing imports this module.

pub mod config;

use std::sync::Arc;
use std::time::Duration as StdDuration;

use axum::Json;
use axum::Router;
use axum::http::Request;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use fred::prelude::{ClientLike, Config as RedisConfig, PerformanceConfig, Pool as RedisPool, ReconnectPolicy};
use jiff::{SignedDuration, Timestamp};
use secrecy::ExposeSecret;
use tokio_util::sync::CancellationToken;
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::files::application::FilesService;
use crate::files::application::permissions::CATALOGUE as FILES_CATALOGUE;
use crate::files::infrastructure::http as files_http;
use crate::files::infrastructure::persistence::ToastyFileRepository;
use crate::files::infrastructure::storage::{LocalFileStorage, S3FileStorage, S3Settings, Storages};
use crate::identity::application::IdentityService;
use crate::identity::application::permissions::CATALOGUE as IDENTITY_CATALOGUE;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::domain::{Catalogue, IdentityError};
use crate::identity::infrastructure::http as identity_http;
use crate::identity::infrastructure::keycloak::{
    HttpRealmAdmin, HttpTokenEndpoint, JwksTokenVerifier, KeycloakSettings, SharedRefreshes,
};
use crate::identity::infrastructure::persistence::{
    ToastyPermissionRegistry, ToastyRoleRepository, ToastyUserRepository,
};
use crate::identity::infrastructure::session_store::RedisSessionStore;
use crate::keycloak_account::application::KeycloakAccountService;
use crate::keycloak_account::infrastructure::http as keycloak_account_http;
use crate::keycloak_account::infrastructure::keycloak::HttpAccountApi;
use crate::shared_http::{problem, timeout};
use crate::shared_kernel::messages::Messages;
use crate::translations::application::TranslationsService;
use crate::translations::application::permissions::CATALOGUE as TRANSLATIONS_CATALOGUE;
use crate::translations::infrastructure::defaults;
use crate::translations::infrastructure::http as translations_http;
use crate::translations::infrastructure::persistence::ToastyTranslationRepository;
use config::AppConfig;

/// A pure type-level bundle: it only names which adapter satisfies each
/// port. Adding a port means one associated type here — nothing else.
#[derive(Clone)]
pub struct ProductionPorts;

impl IdentityPorts for ProductionPorts {
    type Users = ToastyUserRepository;
    type Roles = ToastyRoleRepository;
    type Permissions = ToastyPermissionRegistry;
    type Tokens = HttpTokenEndpoint;
    type Realm = HttpRealmAdmin;
    type Verifier = JwksTokenVerifier;
}

/// What every module checks and the roles they ship with, registered with
/// the identity module at start-up.
pub const CATALOGUES: [Catalogue; 3] = [IDENTITY_CATALOGUE, FILES_CATALOGUE, TRANSLATIONS_CATALOGUE];

/// The texts of one area of the front-end's pages, which live here so that an
/// administrator may edit them too.
macro_rules! front_end {
    ($area:literal) => {
        Messages {
            owner: concat!("front-end ", $area),
            pl: include_str!(concat!("../../translations/", $area, "/pl.json")),
            en: include_str!(concat!("../../translations/", $area, "/en.json")),
        }
    };
}

pub const MESSAGES: [Messages; 12] = [
    crate::shared_http::MESSAGES,
    crate::identity::MESSAGES,
    crate::keycloak_account::MESSAGES,
    crate::files::MESSAGES,
    crate::translations::MESSAGES,
    front_end!("layout"),
    front_end!("common"),
    front_end!("home"),
    front_end!("account"),
    front_end!("files"),
    front_end!("admin"),
    front_end!("legal"),
];

pub type AppIdentityService = IdentityService<ProductionPorts>;

pub type AppKeycloakAccountService = KeycloakAccountService<HttpAccountApi>;

pub type AppFilesService = FilesService<ToastyFileRepository, Storages>;

pub type AppTranslationsService = TranslationsService<ToastyTranslationRepository>;

pub struct Services {
    pub identity: Arc<AppIdentityService>,
    pub keycloak_account: Arc<AppKeycloakAccountService>,
    pub files: Arc<AppFilesService>,
    /// The local disk, which also serves its signed links under `/uploads`.
    pub local_files: LocalFileStorage,
    pub translations: Arc<AppTranslationsService>,
}

pub fn build_services(db: &toasty::Db, config: &AppConfig, redis: RedisPool) -> anyhow::Result<Services> {
    let keycloak = keycloak_settings(config);

    let identity: AppIdentityService = IdentityService::new(
        ToastyUserRepository::new(db.clone()),
        ToastyRoleRepository::new(db.clone()),
        ToastyPermissionRegistry::new(db.clone()),
        HttpTokenEndpoint::new(
            keycloak.clone(),
            SharedRefreshes::new(redis, &config.sessions.redis_key_prefix),
        )?,
        HttpRealmAdmin::new(keycloak.clone())?,
        JwksTokenVerifier::new(&keycloak)?,
    );

    let keycloak_account = KeycloakAccountService::new(HttpAccountApi::new(&keycloak.realm_url)?);

    let local_files = LocalFileStorage::new(
        &config.files.upload_dir,
        &config.files.base_url,
        config.files.signing_key.clone(),
    );
    let s3 = &config.files.s3;
    let object_store = S3FileStorage::new(&S3Settings {
        endpoint: s3.endpoint.clone(),
        public_endpoint: s3.public_endpoint.clone(),
        region: s3.region.clone(),
        bucket: s3.bucket.clone(),
        access_key: s3.access_key.clone(),
        secret_key: s3.secret_key.clone(),
    });
    let files = FilesService::new(
        ToastyFileRepository::new(db.clone()),
        Storages::new(config.files.storage, local_files.clone(), object_store),
        config.files.max_size,
    );

    let translations = TranslationsService::new(
        ToastyTranslationRepository::new(db.clone()),
        defaults::shipped(&MESSAGES)?,
    );

    Ok(Services {
        identity: Arc::new(identity),
        keycloak_account: Arc::new(keycloak_account),
        files: Arc::new(files),
        local_files,
        translations: Arc::new(translations),
    })
}

/// How long one Redis command may take, including the wait for a connection
/// that is being re-established, before it fails.
const REDIS_COMMAND_TIMEOUT: StdDuration = StdDuration::from_secs(2);

/// The first and the longest wait between two attempts to reconnect to Redis,
/// in milliseconds; each wait doubles the one before.
const REDIS_RECONNECT_DELAY_MS: (u32, u32) = (100, 30_000);

/// How long an API request may take before it is answered `503`: longer than
/// two calls to Keycloak one after another, shorter than the front-end's wait.
const REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(25);

/// Connects to Redis, which holds the browser sessions and coordinates token
/// refreshes, so the application holds no state of its own between requests.
/// A lost connection is re-established for as long as it takes, and no
/// command waits for it longer than `REDIS_COMMAND_TIMEOUT`.
pub async fn redis(config: &AppConfig) -> anyhow::Result<RedisPool> {
    let performance = PerformanceConfig {
        default_command_timeout: REDIS_COMMAND_TIMEOUT,
        ..PerformanceConfig::default()
    };
    let (min_delay, max_delay) = REDIS_RECONNECT_DELAY_MS;
    let reconnect = ReconnectPolicy::new_exponential(0, min_delay, max_delay, 2);
    let pool = RedisPool::new(
        RedisConfig::from_url(config.sessions.redis_url.expose_secret())?,
        Some(performance),
        None,
        Some(reconnect),
        1,
    )?;
    pool.init().await?;
    Ok(pool)
}

/// Assembles the HTTP router. The composition root decides which routers
/// sit behind the auth guard; the modules only provide their routes.
pub fn router(services: &Services, config: &AppConfig, redis: RedisPool) -> Router {
    let auth_layer = from_fn_with_state(
        services.identity.clone(),
        identity_http::authenticate::<ProductionPorts>,
    );

    let sign_in = identity_http::SignInSettings {
        keycloak: keycloak_settings(config),
        post_login_url: config.keycloak.post_login_url.clone(),
    };

    let sessions = SessionManagerLayer::new(RedisSessionStore::new(redis, &config.sessions.redis_key_prefix))
        .with_name(config.sessions.cookie_name())
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(config.sessions.cookie_secure)
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.sessions.inactivity_timeout_seconds,
        )));

    // The account on Keycloak shares `/api/users/me` with the account mirrored here.
    let users_routes = identity_http::users_router(services.identity.clone())
        .merge(keycloak_account_http::account_router(services.keycloak_account.clone()))
        .layer(auth_layer.clone());

    let files_routes = files_http::files_router(services.files.clone()).layer(auth_layer.clone());

    let uploads_routes = files_http::uploads_router(services.local_files.clone());

    let translations_admin_routes =
        translations_http::translations_admin_router(services.translations.clone()).layer(auth_layer.clone());

    let roles_routes = identity_http::roles_router(services.identity.clone()).layer(auth_layer);

    // Every route but the local disk's, whose transfers last as long as the file takes.
    let api_routes = Router::new()
        .nest(
            "/api/auth",
            identity_http::auth_router(services.identity.clone(), sign_in),
        )
        .nest("/api/users", users_routes)
        .nest("/api/files", files_routes)
        .nest(
            "/api/translations",
            translations_http::translations_public_router(services.translations.clone()),
        )
        .nest("/api/admin/translations", translations_admin_routes)
        .nest("/api/admin", roles_routes)
        .route("/health", get(health))
        .layer(from_fn_with_state(REQUEST_TIMEOUT, timeout::within));

    Router::new()
        .merge(api_routes)
        .nest("/uploads", uploads_routes)
        .fallback(problem::not_found)
        .layer(sessions)
        .layer(CatchPanicLayer::custom(problem::panicked))
        .layer(
            // The span names the path only: a query may carry an authorization code or a link's signature.
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                tracing::debug_span!("request", method = %request.method(), path = %request.uri().path())
            }),
        )
}

/// How long an allowed upload may stay pending before it counts as abandoned.
const ABANDONED_AFTER: SignedDuration = SignedDuration::from_hours(6);

/// How often abandoned uploads are looked for.
const SWEEP_EVERY: std::time::Duration = std::time::Duration::from_mins(15);

/// Removes abandoned uploads every `SWEEP_EVERY` until `shutdown` is cancelled.
pub async fn sweep_abandoned_uploads(files: Arc<AppFilesService>, shutdown: CancellationToken) {
    let mut ticks = tokio::time::interval(SWEEP_EVERY);
    loop {
        tokio::select! {
            () = shutdown.cancelled() => return,
            _ = ticks.tick() => {}
        }
        let before = Timestamp::now() - ABANDONED_AFTER;
        match files.sweep_abandoned(before).await {
            Ok(0) => {}
            Ok(removed) => tracing::info!(removed, "abandoned uploads removed"),
            Err(err) => tracing::warn!("abandoned uploads could not be removed: {err}"),
        }
    }
}

pub async fn register_permissions(services: &Services) -> Result<(), IdentityError> {
    services.identity.register(&CATALOGUES).await
}

fn keycloak_settings(config: &AppConfig) -> KeycloakSettings {
    KeycloakSettings {
        realm_url: config.keycloak.realm_url.clone(),
        client_id: config.keycloak.client_id.clone(),
        client_secret: config.keycloak.client_secret.clone(),
        audience: config.keycloak.audience.clone(),
        callback_url: config.keycloak.callback_url.clone(),
    }
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "UP" }))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{CATALOGUES, MESSAGES};
    use crate::translations::domain::icu;
    use crate::translations::infrastructure::defaults::shipped;

    #[test]
    fn both_languages_of_every_shipped_message_use_the_same_placeholders() {
        let catalogue = shipped(&MESSAGES).unwrap();

        let differing: Vec<&String> = catalogue
            .iter()
            .filter(|(_, text)| icu::placeholders(&text.pl) != icu::placeholders(&text.en))
            .map(|(key, _)| key)
            .collect();

        assert_eq!(differing, Vec::<&String>::new());
    }

    #[test]
    fn every_permission_has_one_module_and_every_stock_grant_is_declared() {
        let mut names = BTreeSet::new();
        for permission in CATALOGUES.iter().flat_map(|catalogue| catalogue.permissions) {
            assert!(names.insert(permission.name), "{} is declared twice", permission.name);
        }
        for grant in CATALOGUES
            .iter()
            .flat_map(|catalogue| catalogue.stock_roles)
            .flat_map(|role| role.grants)
        {
            assert!(names.contains(grant), "{grant} is granted but not declared");
        }
    }
}
