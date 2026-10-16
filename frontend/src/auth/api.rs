use std::collections::BTreeSet;

use jiff::Timestamp;
use reqwest::Method;
use serde::Deserialize;
use topcoat::context::Cx;

use crate::backend::{self, BackendError};

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<String>,
    pub created_at: Timestamp,
}

impl User {
    /// The full name, or the email address when Keycloak holds no name.
    #[must_use]
    pub fn display_name(&self) -> String {
        let name = format!("{} {}", self.first_name, self.last_name);
        let name = name.trim();
        if name.is_empty() {
            self.email.clone()
        } else {
            name.to_owned()
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Grants {
    pub unrestricted: bool,
    pub permissions: BTreeSet<String>,
}

pub async fn me(cx: &Cx) -> Result<User, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/users/me")).await
}

pub async fn my_grants(cx: &Cx) -> Result<Grants, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/users/me/permissions")).await
}

pub async fn sign_out(cx: &Cx) -> Result<(), BackendError> {
    backend::send(cx, backend::request(cx, Method::POST, "/api/auth/logout")).await?;
    Ok(())
}
