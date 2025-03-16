use crate::auth::auth_module;
use crate::auth::middleware::jwt_secret_middleware::jwt_secret_middleware;
use crate::common::middleware::i18n_middleware::i18n_middleware;
use crate::common::r#struct::app_state::AppState;
use crate::files::files_module;
use crate::users::users_module;
use axum::{Router, middleware::from_fn};
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

pub async fn configure(app_state: AppState) -> Router {
	let jwt_access_token_secret = app_state.config.security.tokens.jwt_access_token.secret.clone();

	Router::new()
		.merge(users_module::configure(app_state.clone()))
		.merge(auth_module::configure(app_state.clone()))
		.merge(files_module::configure(app_state.clone()))
		.layer(TraceLayer::new_for_http())
		.layer(CookieManagerLayer::new())
		.layer(from_fn(i18n_middleware))
		.layer(from_fn(move |req, next| {
			let jwt_secret = jwt_access_token_secret.clone();
			jwt_secret_middleware(jwt_secret, req, next)
		}))
}
