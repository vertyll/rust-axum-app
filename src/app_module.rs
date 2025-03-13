use crate::auth::auth_module;
use crate::users::users_module;
use axum::Router;
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub async fn configure(db_pool: PgPool, jwt_secret: String) -> Router {
	Router::new()
		// Add the users module
		.merge(users_module::configure(db_pool.clone()))
		// Add the auth module
		.merge(auth_module::configure(db_pool.clone(), jwt_secret))
		// Add middleware for tracing HTTP requests
		.layer(TraceLayer::new_for_http())
}
