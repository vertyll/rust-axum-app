use axum::Router;
use axum::middleware::from_fn_with_state;

use crate::auth::middleware::jwt_auth::auth_middleware;
use crate::common::r#struct::state::AppState;
use crate::users::controllers::users_controller;

pub fn configure(app_state: AppState) -> Router {
	Router::new()
		.nest("/api/users",
			  users_controller::routes(app_state.db.clone())
				  .layer(from_fn_with_state(app_state.jwt_access_token_secret.clone(), auth_middleware))
		)
}