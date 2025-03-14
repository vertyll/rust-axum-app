use axum::Router;
use sea_orm::DatabaseConnection;

use crate::auth::controllers::auth_controller;

pub fn configure(db: DatabaseConnection, jwt_access_token_secret: String, jwt_access_token_expires_in: i64) -> Router {
	Router::new().nest("/api/auth", auth_controller::routes(db, jwt_access_token_secret, jwt_access_token_expires_in))
}
