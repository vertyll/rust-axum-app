use std::sync::Arc;

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use validator::Validate;

use super::Identity;
use crate::identity::application::permissions::UsersView;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::UserId;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::{Auth, Permitted};
use crate::identity::infrastructure::http::responses::{GrantsResponse, UserResponse};
use crate::shared_http::pagination::{PageQuery, PageResponse};
use crate::shared_http::problem::{PathParam, QueryParams};

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

async fn my_permissions(Auth(caller): Auth) -> Json<GrantsResponse> {
    Json(GrantsResponse::from(&caller.grants))
}

async fn list_users<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersView>,
    QueryParams(query): QueryParams<PageQuery>,
) -> Result<Json<PageResponse<UserResponse>>, ApiError> {
    query.validate()?;
    Ok(Json(identity.list_users(query.request()).await?.into()))
}

async fn get_user<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersView>,
    PathParam(id): PathParam<i64>,
) -> Result<Json<UserResponse>, ApiError> {
    Ok(Json(identity.get_user(UserId(id)).await?.into()))
}
