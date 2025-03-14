use axum::Router;

use crate::common::r#struct::app_state::AppState;
use crate::users::controllers::users_controller;

pub fn configure(app_state: AppState) -> Router {
	Router::new().nest("/api/users", users_controller::routes(app_state.clone()))
}
