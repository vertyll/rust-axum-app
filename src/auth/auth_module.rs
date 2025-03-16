use axum::Router;

use crate::auth::controllers::auth_controller;
use crate::common::r#struct::app_state::AppState;
use crate::common::r#struct::token_state::TokenState;

pub fn configure(app_state: AppState, token_state: TokenState) -> Router {
	Router::new().nest(
		"/api/auth",
		auth_controller::routes(app_state.clone(), token_state.clone()),
	)
}
