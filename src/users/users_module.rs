use axum::Router;
use sea_orm::DatabaseConnection;

use crate::users::controllers::users_controller;

pub fn configure(db: DatabaseConnection) -> Router {
	Router::new().nest("/api/users", users_controller::routes(db))
}
