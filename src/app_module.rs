use crate::auth::auth_module;
use crate::auth::middleware::jwt_secret_middleware::jwt_secret_middleware;
use crate::common::middleware::i18n_middleware::i18n_middleware;
use crate::config::app_config::AppConfig;
use crate::files::files_module;
use crate::users::users_module;
use axum::{Extension, Router, middleware::from_fn};
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use crate::di::module::AppModule;

pub async fn configure(config: Arc<AppConfig>, di_module: Arc<AppModule>) -> Router {
	let jwt_access_token_secret = config.security.tokens.jwt_access_token.secret.clone();

	Router::new()
		// Add all modules
		.merge(users_module::configure())
		.merge(auth_module::configure())
		.merge(files_module::configure())
		.layer(TraceLayer::new_for_http())
		.layer(CookieManagerLayer::new())
		// Add important dependencies and configurations to the app
		.layer(Extension(config.clone()))
		.layer(Extension(di_module.users_service.clone()))
		.layer(Extension(di_module.auth_service.clone()))
		.layer(Extension(di_module.refresh_token_service.clone()))
		.layer(Extension(di_module.email_service.clone()))
		.layer(Extension(di_module.user_roles_service.clone()))
		.layer(Extension(di_module.confirmation_token_service.clone()))
		.layer(Extension(di_module.files_service.clone()))
		.layer(Extension(di_module.roles_service.clone()))
		.layer(from_fn(i18n_middleware))
		.layer(from_fn(move |req, next| {
			let jwt_secret = jwt_access_token_secret.clone();
			jwt_secret_middleware(jwt_secret, req, next)
		}))
}
