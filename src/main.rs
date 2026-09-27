use std::sync::Arc;
use std::time::Duration;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use rust_axum_app::bootstrap::{self, AppIdentityService, config::AppConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();

	tracing_subscriber::registry()
		.with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,tower_http=debug".into()))
		.with(tracing_subscriber::fmt::layer())
		.init();

	let config = AppConfig::load()?;
	config.log_summary();

	let db = toasty::Db::builder()
		.models(toasty::models!(rust_axum_app::*))
		.max_pool_size(config.database.max_connections as usize)
		.connect(&config.database.url())
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

	bootstrap::seed::seed_roles(&db).await?;

	let services = bootstrap::build_services(db, &config)?;
	services.translations.synchronize().await?;
	spawn_session_cleanup(services.identity.clone());

	let app = bootstrap::router(&services, &config);
	let address = format!("{}:{}", config.server.host, config.server.port);
	let listener = tokio::net::TcpListener::bind(&address).await?;
	tracing::info!("listening on http://{address}");

	axum::serve(listener, app)
		.with_graceful_shutdown(shutdown_signal())
		.await?;
	Ok(())
}

/// Deletes expired refresh sessions once a day.
fn spawn_session_cleanup(identity: Arc<AppIdentityService>) {
	tokio::spawn(async move {
		let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
		loop {
			interval.tick().await;
			match identity.clean_expired_sessions().await {
				Ok(()) => tracing::debug!("expired refresh sessions cleaned"),
				Err(err) => tracing::error!("session cleanup failed: {err}"),
			}
		}
	});
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
