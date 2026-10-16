//! The migration command line, after the Toasty guide; its commands are in
//! `docs/development-setup.md`.

use secrecy::ExposeSecret;
use toasty_cli::{Config, ToastyCli};

use rust_axum_app_backend::bootstrap::config::AppConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let app_config = AppConfig::load()?;
    let cli_config = Config::load()?;

    let db = toasty::Db::builder()
        .models(toasty::models!(rust_axum_app_backend::*))
        .connect(app_config.database.url().expose_secret())
        .await?;

    ToastyCli::with_config(db, cli_config).parse_and_run().await?;
    Ok(())
}
