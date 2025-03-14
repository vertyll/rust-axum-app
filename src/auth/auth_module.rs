use axum::Router;
use sea_orm::DatabaseConnection;

use crate::auth::controllers::auth_controller;

pub fn configure(db: DatabaseConnection) -> Router {
	Router::new().nest("/api/auth", auth_controller::routes(db))
}
