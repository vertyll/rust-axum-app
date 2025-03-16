use axum::Router;

use crate::common::r#struct::app_state::AppState;
use crate::files::controllers::files_controller;

pub fn configure(app_state: AppState) -> Router {
	Router::new().nest("/api/files", files_controller::routes(app_state.clone()))
}
