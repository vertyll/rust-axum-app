use axum::Router;

use crate::auth::controllers::auth_controller;
use crate::common::r#struct::app_state::AppState;

pub fn configure(app_state: AppState) -> Router {
	Router::new().nest("/api/auth", auth_controller::routes(app_state.clone()))
}
