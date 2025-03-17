use axum::Router;

use crate::auth::controllers::auth_controller;

pub fn configure() -> Router {
	Router::new().nest("/api/auth", auth_controller::routes())
}
