//! The server-rendered front-end, on Topcoat. It holds no token and no session
//! of its own: the back-end signs people in, and this application forwards the
//! browser's session cookie with every call. The modules are laid out in
//! `frontend/README.md`.

mod app;
mod auth;
mod backend;
mod components;
mod config;
mod forms;
mod i18n;

use topcoat::asset::{AssetBundle, RouterBuilderAssetExt};
use topcoat::cookie::RouterBuilderCookieExt;
use topcoat::runtime::RouterBuilderRuntimeExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::backend::Backend;
use crate::config::AppConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    tracing::info!(
        environment = ?config.environment,
        backend = %config.backend_url,
        "configuration loaded"
    );

    let router = app::router()
        .route(backend::proxy::api)
        .route(backend::proxy::uploads)
        .cookies()
        .assets(AssetBundle::load()?)
        .runtime()
        .app_context(Backend::new(&config.backend_url, config.backend_timeout)?)
        .app_context(config)
        .build();

    topcoat::start(router).await?;
    Ok(())
}
