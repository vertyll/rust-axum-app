use axum::Router;
use sqlx::PgPool;

use crate::auth::controllers::auth_controller;

pub fn configure(db_pool: PgPool) -> Router {
	Router::new().nest("/api/auth", auth_controller::routes(db_pool))
}
