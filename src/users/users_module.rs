use crate::users::controllers::users_controller;
use axum::Router;

pub fn configure() -> Router {
	Router::new().nest("/api/users", users_controller::routes())
}
