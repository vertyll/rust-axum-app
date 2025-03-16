use crate::config::app_config::AppConfig;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
	pub db: DatabaseConnection,
	pub config: Arc<AppConfig>,
}

impl AppState {
	pub fn new(db: DatabaseConnection, config: Arc<AppConfig>) -> Self {
		Self { db, config }
	}
}
