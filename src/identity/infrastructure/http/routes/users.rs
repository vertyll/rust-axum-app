use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

use super::Identity;
use crate::identity::application::permissions::UsersView;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::UserId;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::{Auth, Permitted};
use crate::identity::infrastructure::http::responses::{GrantsResponse, UserResponse};
use crate::shared_infrastructure::problem::PathParam;

pub fn users_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(me))
        .route("/me/permissions", get(my_permissions))
        .route("/{id}", get(get_user))
        .with_state(identity)
}

async fn me<P: IdentityPorts>(
    State(identity): Identity<P>,
    Auth(caller): Auth,
) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(identity.get_user(caller.user_id).await?.into()))
}

/// What the caller's roles grant, so a client shows only what they may do.
async fn my_permissions(Auth(caller): Auth) -> Json<GrantsResponse> {
    Json(GrantsResponse::from(&caller.grants))
}

async fn list_users<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersView>,
) -> Result<Json<Vec<UserResponse>>, ApiError> {
    let users = identity.list_users().await?;
    Ok(Json(users.into_iter().map(UserResponse::from).collect()))
}

async fn get_user<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersView>,
    PathParam(id): PathParam<i64>,
) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(identity.get_user(UserId(id)).await?.into()))
}
