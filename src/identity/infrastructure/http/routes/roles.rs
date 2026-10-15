//! Roles and what they grant, and who holds which role. The composition
//! root nests these under `/api/admin`, behind the auth layer.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use validator::Validate;

use super::Identity;
use crate::identity::application::permissions::{RolesManage, RolesView, UsersManage};
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::UserId;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::Permitted;
use crate::identity::infrastructure::http::requests::{NewRoleRequest, RoleChangesRequest};
use crate::identity::infrastructure::http::responses::{ModuleResponse, RoleResponse};
use crate::shared_infrastructure::problem::{JsonBody, PathParam};

pub fn roles_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
    Router::new()
        .route("/permissions", get(permissions))
        .route("/roles", get(list_roles).post(create_role))
        .route("/roles/{name}", get(get_role).put(update_role).delete(delete_role))
        .route("/users/{id}/roles/{name}", put(assign_role).delete(remove_role))
        .with_state(identity)
}

async fn permissions<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesView>,
) -> Result<Json<Vec<ModuleResponse>>, ApiError> {
    Ok(Json(ModuleResponse::grouped(identity.permissions().await?)))
}

async fn list_roles<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesView>,
) -> Result<Json<Vec<RoleResponse>>, ApiError> {
    let roles = identity.list_roles().await?;
    Ok(Json(roles.into_iter().map(RoleResponse::from).collect()))
}

async fn get_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesView>,
    PathParam(name): PathParam<String>,
) -> Result<Json<RoleResponse>, ApiError> {
    Ok(Json(identity.get_role(&name).await?.into()))
}

async fn create_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesManage>,
    JsonBody(request): JsonBody<NewRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), ApiError> {
    request.validate()?;
    let role = identity.create_role(request.into()).await?;
    Ok((StatusCode::CREATED, Json(role.into())))
}

async fn update_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesManage>,
    PathParam(name): PathParam<String>,
    JsonBody(request): JsonBody<RoleChangesRequest>,
) -> Result<Json<RoleResponse>, ApiError> {
    request.validate()?;
    Ok(Json(identity.update_role(&name, request.into()).await?.into()))
}

async fn delete_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<RolesManage>,
    PathParam(name): PathParam<String>,
) -> Result<StatusCode, ApiError> {
    identity.delete_role(&name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn assign_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersManage>,
    PathParam((id, name)): PathParam<(i64, String)>,
) -> Result<StatusCode, ApiError> {
    identity.assign_role(UserId(id), &name).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_role<P: IdentityPorts>(
    State(identity): Identity<P>,
    Permitted(_caller, _): Permitted<UsersManage>,
    PathParam((id, name)): PathParam<(i64, String)>,
) -> Result<StatusCode, ApiError> {
    identity.remove_role(UserId(id), &name).await?;
    Ok(StatusCode::NO_CONTENT)
}
