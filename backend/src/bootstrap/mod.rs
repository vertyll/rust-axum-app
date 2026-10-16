//! Composition root — the only place naming concrete adapters. Dependency
//! injection is constructor calls plus the aliases below, all resolved at
//! compile time; nothing imports this module.

pub mod config;

use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use fred::prelude::{ClientLike, Config as RedisConfig, Pool as RedisPool};
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::cookie::time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer};

use crate::files::application::FilesService;
use crate::files::application::permissions::CATALOGUE as FILES_CATALOGUE;
use crate::files::infrastructure::http as files_http;
use crate::files::infrastructure::persistence::ToastyFileRepository;
use crate::files::infrastructure::storage::LocalFileStorage;
use crate::identity::CATALOGUE as IDENTITY_CATALOGUE;
use crate::identity::application::IdentityService;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::domain::{Catalogue, IdentityError};
use crate::identity::infrastructure::http as identity_http;
use crate::identity::infrastructure::keycloak::{
    HttpKeycloakClient, HttpRealmAdmin, JwksTokenVerifier, KeycloakSettings, SharedRefreshes,
};
use crate::identity::infrastructure::persistence::{ToastyRoleRepository, ToastyUserRepository};
use crate::identity::infrastructure::session_store::RedisSessionStore;
use crate::shared_http::problem;
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
    type Keycloak = HttpKeycloakClient;
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

/// The default messages of every module and of every area of the front-end.
pub const MESSAGES: [Messages; 11] = [
    crate::shared_http::MESSAGES,
    crate::identity::MESSAGES,
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

pub type AppFilesService = FilesService<ToastyFileRepository, LocalFileStorage>;

pub type AppTranslationsService = TranslationsService<ToastyTranslationRepository>;

pub struct Services {
    pub identity: Arc<AppIdentityService>,
    pub files: Arc<AppFilesService>,
    pub translations: Arc<AppTranslationsService>,
}

pub fn build_services(db: &toasty::Db, config: &AppConfig, redis: RedisPool) -> anyhow::Result<Services> {
    let keycloak = keycloak_settings(config);

    let identity: AppIdentityService = IdentityService::new(
        ToastyUserRepository::new(db.clone()),
        ToastyRoleRepository::new(db.clone()),
        HttpKeycloakClient::new(
            keycloak.clone(),
            SharedRefreshes::new(redis, &config.sessions.redis_key_prefix),
        )?,
        HttpRealmAdmin::new(keycloak.clone())?,
        JwksTokenVerifier::new(&keycloak)?,
    );

    let files = FilesService::new(
        ToastyFileRepository::new(db.clone()),
        LocalFileStorage::new(&config.files.upload_dir, &config.files.base_url),
    );

    let translations = TranslationsService::new(
        ToastyTranslationRepository::new(db.clone()),
        defaults::shipped(&MESSAGES)?,
    );

    Ok(Services {
        identity: Arc::new(identity),
        files: Arc::new(files),
        translations: Arc::new(translations),
    })
}

/// Connects to Redis, which holds the browser sessions and coordinates token
/// refreshes, so the application holds no state of its own between requests.
pub async fn redis(config: &AppConfig) -> anyhow::Result<RedisPool> {
    let pool = RedisPool::new(RedisConfig::from_url(&config.sessions.redis_url)?, None, None, None, 1)?;
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
        .with_name("RUST_AXUM_APP_SESSION")
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(config.sessions.cookie_secure)
        .with_path("/")
        .with_expiry(Expiry::OnInactivity(Duration::seconds(
            config.sessions.inactivity_timeout_seconds,
        )));

    let users_routes = identity_http::users_router(services.identity.clone()).layer(auth_layer.clone());

    let files_routes = files_http::files_router(services.files.clone()).layer(auth_layer.clone());

    let uploads_routes = files_http::uploads_router(&config.files.upload_dir).layer(auth_layer.clone());

    let translations_admin_routes =
        translations_http::translations_admin_router(services.translations.clone()).layer(auth_layer.clone());

    let roles_routes = identity_http::roles_router(services.identity.clone()).layer(auth_layer);

    Router::new()
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
        .nest("/uploads", uploads_routes)
        .route("/health", get(health))
        .fallback(problem::not_found)
        .layer(sessions)
        .layer(TraceLayer::new_for_http())
}

/// Registers what every module checks, so roles can grant it.
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
