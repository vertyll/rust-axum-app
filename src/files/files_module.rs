use axum::Router;

use crate::files::controllers::files_controller;

pub fn configure() -> Router {
	Router::new().nest("/api/files", files_controller::routes())
}
