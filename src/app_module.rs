use crate::auth::auth_module;
use crate::auth::middleware::jwt_secret_middleware::jwt_secret_middleware;
use crate::common::middleware::i18n_middleware::i18n_middleware;
use crate::common::r#struct::app_state::AppState;
use crate::users::users_module;
use axum::{Router, middleware::from_fn};
use tower_http::trace::TraceLayer;

pub async fn configure(app_state: AppState, jwt_access_token_expires_in: i64) -> Router {
	let jwt_access_token_secret = app_state.jwt_access_token_secret.clone();

	Router::new()
		// Add the users module
		.merge(users_module::configure(app_state.clone()))
		// Add the auth module
		.merge(auth_module::configure(app_state.clone(), jwt_access_token_expires_in))
		// Add middleware for tracing HTTP requests
		.layer(TraceLayer::new_for_http())
		// Add i18n middleware
		.layer(from_fn(i18n_middleware))
		// Add JWT secret middleware
		.layer(from_fn(move |req, next| {
			let jwt_secret = jwt_access_token_secret.clone();
			jwt_secret_middleware(jwt_secret, req, next)
		}))
}
