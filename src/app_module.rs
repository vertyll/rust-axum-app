use crate::auth::auth_module;
use crate::common::middleware::i18n::i18n_middleware;
use crate::users::users_module;
use axum::{Router, middleware::from_fn};
use tower_http::trace::TraceLayer;
use crate::common::r#struct::state::AppState;

pub async fn configure(app_state: AppState, jwt_access_token_expires_in: i64) -> Router {
	Router::new()
		// Add the users module
		.merge(users_module::configure(app_state.clone()))
		// Add the auth module
		.merge(auth_module::configure(app_state.clone(), jwt_access_token_expires_in))
		// Add middleware for tracing HTTP requests
		.layer(TraceLayer::new_for_http())
		// Add i18n middleware to handle language selection
		.layer(from_fn(i18n_middleware))
}
