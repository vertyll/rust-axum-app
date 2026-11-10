use std::time::Duration;

use secrecy::ExposeSecret;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use rust_axum_app_backend::bootstrap::{self, config::AppConfig};

/// How long a query may wait for a free connection, and a new connection may
/// take to open, before it fails.
const DB_POOL_TIMEOUT: Duration = Duration::from_secs(5);

/// How long a connection is reused before it is replaced, shorter than the
/// idle timeouts of the network between here and the database.
const DB_CONNECTION_LIFETIME: Duration = Duration::from_mins(30);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load()?;
    config.log_summary();

    let db = toasty::Db::builder()
        .models(toasty::models!(rust_axum_app_backend::*))
        .max_pool_size(config.database.max_connections.get() as usize)
        .pool_wait_timeout(Some(DB_POOL_TIMEOUT))
        .pool_create_timeout(Some(DB_POOL_TIMEOUT))
        .pool_max_connection_lifetime(Some(DB_CONNECTION_LIFETIME))
        .connect(config.database.url().expose_secret())
        .await?;

    if config.server.environment.is_local()
        && let Err(err) = db.push_schema().await
    {
        if err.to_string().contains("already exists") {
            tracing::debug!("schema already present, push_schema skipped");
        } else {
            tracing::warn!("push_schema failed ({err}); run `cargo run --bin cli -- migration apply`");
        }
    }

    let redis = bootstrap::redis(&config).await?;
    let services = bootstrap::build_services(&db, &config, redis.clone())?;
    bootstrap::register_permissions(&services).await?;
    services.translations.synchronize().await?;

    let shutdown = CancellationToken::new();
    let sweeper = tokio::spawn(bootstrap::sweep_abandoned_uploads(
        services.files.clone(),
        shutdown.clone(),
    ));

    let app = bootstrap::router(&services, &config, redis);
    let address = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&address).await?;
    tracing::info!("listening on http://{address}");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    shutdown.cancel();
    sweeper.await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
