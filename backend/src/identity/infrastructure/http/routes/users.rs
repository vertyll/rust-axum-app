use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use validator::Validate;

use super::Identity;
use crate::identity::application::permissions::UsersView;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::UserId;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::{Auth, CallerToken, Permitted};
use crate::identity::infrastructure::http::requests::AccountLanguageRequest;
use crate::identity::infrastructure::http::responses::{GrantsResponse, SecondFactorResponse, UserResponse};
use crate::shared_http::pagination::{PageQuery, PageResponse};
use crate::shared_http::problem::{JsonBody, PathParam, QueryParams};

pub fn users_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/me", get(me))
        .route("/me/permissions", get(my_permissions))
        .route("/me/second-factor", get(my_second_factor))
        .route("/me/language", put(keep_my_language))
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

/// Stores the language the reader chose here on their Keycloak account, which Keycloak's pages show a signed-in
/// person in.
async fn keep_my_language<P: IdentityPorts>(
    State(identity): Identity<P>,
    Auth(caller): Auth,
    JsonBody(request): JsonBody<AccountLanguageRequest>,
) -> Result<StatusCode, ApiError> {
    request.validate()?;
    identity
        .keep_account_language(&caller.keycloak_id, &request.language)
        .await;
    Ok(StatusCode::NO_CONTENT)
}

async fn my_second_factor<P: IdentityPorts>(
    State(identity): Identity<P>,
    CallerToken(token): CallerToken,
) -> Result<Json<SecondFactorResponse>, ApiError> {
    Ok(Json(identity.second_factor(&token).await?.into()))
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
