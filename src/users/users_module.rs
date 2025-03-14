use axum::Router;
use axum::middleware::from_fn_with_state;
use sea_orm::DatabaseConnection;

use crate::auth::middleware::jwt_auth::auth_middleware;
use crate::users::controllers::users_controller;

pub fn configure(db: DatabaseConnection, jwt_access_token_secret: String) -> Router {
	Router::new()
		.nest("/api/users",
			  users_controller::routes(db)
				  .layer(from_fn_with_state(jwt_access_token_secret, auth_middleware))
		)
}