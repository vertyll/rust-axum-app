use axum::Router;
use sqlx::PgPool;

use crate::users::controllers::users_controller;

pub fn configure(db_pool: PgPool) -> Router {
	Router::new().nest("/api/users", users_controller::routes(db_pool))
}
