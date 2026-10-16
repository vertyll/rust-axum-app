use reqwest::Method;
use topcoat::context::Cx;

use crate::auth::api::User;
use crate::backend::{self, BackendError, Page, PageRequest, segment};

pub async fn users(cx: &Cx, request: PageRequest) -> Result<Page<User>, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/users").query(&request)).await
}

pub async fn user(cx: &Cx, id: i64) -> Result<User, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, &format!("/api/users/{id}"))).await
}

pub async fn assign_role(cx: &Cx, user_id: i64, role: &str) -> Result<(), BackendError> {
    let path = format!("/api/admin/users/{user_id}/roles/{}", segment(role));
    backend::send(cx, backend::request(cx, Method::PUT, &path)).await?;
    Ok(())
}

pub async fn remove_role(cx: &Cx, user_id: i64, role: &str) -> Result<(), BackendError> {
    let path = format!("/api/admin/users/{user_id}/roles/{}", segment(role));
    backend::send(cx, backend::request(cx, Method::DELETE, &path)).await?;
    Ok(())
}
