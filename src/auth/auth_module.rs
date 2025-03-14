use axum::Router;

use crate::auth::controllers::auth_controller;
use crate::common::r#struct::app_state::AppState;

pub fn configure(app_state: AppState, jwt_access_token_expires_in: i64) -> Router {
	Router::new().nest(
		"/api/auth",
		auth_controller::routes(app_state.clone(), jwt_access_token_expires_in),
	)
}
