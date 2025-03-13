use crate::auth::auth_module;
use crate::common::middleware::i18n::i18n_middleware;
use crate::users::users_module;
use axum::{Router, middleware::from_fn};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub async fn configure(db_pool: PgPool) -> Router {
	Router::new()
		// Add the users module
		.merge(users_module::configure(db_pool.clone()))
		// Add the auth module
		.merge(auth_module::configure(db_pool.clone()))
		// Add middleware for tracing HTTP requests
		.layer(TraceLayer::new_for_http())
		// Add i18n middleware to handle language selection
		.layer(from_fn(i18n_middleware))
}
