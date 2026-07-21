//! Migration CLI (`cargo run --bin cli -- <command>`), following the Toasty
//! guide's per-project CLI pattern:
//!
//! ```text
//! cargo run --bin cli -- migration generate --name init
//! cargo run --bin cli -- migration apply
//! cargo run --bin cli -- snapshot
//! ```
//!
//! Migration files live in `toasty/` (see `Toasty.toml`).

use toasty_cli::{Config, ToastyCli};

use rust_axum_app::bootstrap::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	dotenvy::dotenv().ok();

	let app_config = AppConfig::load()?;
	let cli_config = Config::load()?;

	let db = toasty::Db::builder()
		.models(toasty::models!(rust_axum_app::*))
		.connect(&app_config.database.url())
		.await?;

	ToastyCli::with_config(db, cli_config).parse_and_run().await?;
	Ok(())
}
