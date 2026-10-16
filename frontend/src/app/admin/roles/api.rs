use std::collections::BTreeSet;

use jiff::Timestamp;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

use crate::backend::{self, BackendError, Page, PageRequest, segment};

#[derive(Debug, Clone, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Module {
    pub module: String,
    pub permissions: Vec<DeclaredPermission>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeclaredPermission {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct NewRole<'a> {
    pub name: &'a str,
    pub description: &'a str,
}

#[derive(Debug, Serialize)]
pub struct RoleChanges<'a> {
    pub description: &'a str,
    pub unrestricted: bool,
    pub permissions: &'a BTreeSet<String>,
}

pub async fn roles(cx: &Cx, request: PageRequest) -> Result<Page<Role>, BackendError> {
    backend::json(
        cx,
        backend::request(cx, Method::GET, "/api/admin/roles").query(&request),
    )
    .await
}

pub async fn role(cx: &Cx, name: &str) -> Result<Role, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, &role_path(name))).await
}

pub async fn permissions(cx: &Cx) -> Result<Vec<Module>, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/admin/permissions")).await
}

pub async fn create_role(cx: &Cx, role: &NewRole<'_>) -> Result<Role, BackendError> {
    backend::json(cx, backend::request(cx, Method::POST, "/api/admin/roles").json(role)).await
}

pub async fn update_role(cx: &Cx, name: &str, changes: &RoleChanges<'_>) -> Result<Role, BackendError> {
    backend::json(cx, backend::request(cx, Method::PUT, &role_path(name)).json(changes)).await
}

pub async fn delete_role(cx: &Cx, name: &str) -> Result<(), BackendError> {
    backend::send(cx, backend::request(cx, Method::DELETE, &role_path(name))).await?;
    Ok(())
}

fn role_path(name: &str) -> String {
    format!("/api/admin/roles/{}", segment(name))
}
