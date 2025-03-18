use crate::auth::auth_module;
use crate::auth::middleware::jwt_secret_middleware::jwt_secret_middleware;
use crate::auth::services::auth_service::IAuthService;
use crate::auth::services::confirmation_token_service::IConfirmationTokenService;
use crate::auth::services::refresh_token_service::IRefreshTokenService;
use crate::common::middleware::i18n_middleware::i18n_middleware;
use crate::config::app_config::AppConfig;
use crate::di::module::AppModule;
use crate::emails::services::emails_service::IEmailsService;
use crate::files::files_module;
use crate::files::services::files_service::IFilesService;
use crate::roles::services::roles_service::IRolesService;
use crate::roles::services::user_roles_service::IUserRolesService;
use crate::users::services::users_service::IUsersService;
use crate::users::users_module;
use axum::{Extension, Router, middleware::from_fn};
use shaku::HasComponent;
use std::sync::Arc;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;

pub async fn configure(config: Arc<AppConfig>, di_module: Arc<AppModule>) -> Router {
	let jwt_access_token_secret = config.security.tokens.jwt_access_token.secret.clone();

	// Resolve all required services
	let users_service: Arc<dyn IUsersService> = di_module.resolve();
	let auth_service: Arc<dyn IAuthService> = di_module.resolve();
	let refresh_token_service: Arc<dyn IRefreshTokenService> = di_module.resolve();
	let email_service: Arc<dyn IEmailsService> = di_module.resolve();
	let user_roles_service: Arc<dyn IUserRolesService> = di_module.resolve();
	let confirmation_token_service: Arc<dyn IConfirmationTokenService> = di_module.resolve();
	let files_service: Arc<dyn IFilesService> = di_module.resolve();
	let roles_service: Arc<dyn IRolesService> = di_module.resolve();

	Router::new()
		// Add all modules
		.merge(users_module::configure())
		.merge(auth_module::configure())
		.merge(files_module::configure())
		.layer(TraceLayer::new_for_http())
		.layer(CookieManagerLayer::new())
		// Add important dependencies and configurations to the app
		.layer(Extension(config.clone()))
		.layer(Extension(users_service))
		.layer(Extension(auth_service))
		.layer(Extension(refresh_token_service))
		.layer(Extension(email_service))
		.layer(Extension(user_roles_service))
		.layer(Extension(confirmation_token_service))
		.layer(Extension(files_service))
		.layer(Extension(roles_service))
		.layer(from_fn(i18n_middleware))
		.layer(from_fn(move |req, next| {
			let jwt_secret = jwt_access_token_secret.clone();
			jwt_secret_middleware(jwt_secret, req, next)
		}))
}
