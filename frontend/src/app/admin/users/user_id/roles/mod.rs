pub(crate) mod remove;

use serde::Deserialize;
use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, path_param, route};

use super::super::api;
use super::UserId;
use crate::auth::{Permission, require_permission};
use crate::forms::{done, refused};

#[derive(Deserialize)]
pub(crate) struct RoleChoice {
    pub(crate) role: String,
}

#[route(POST)]
pub(crate) async fn assign_role(cx: &Cx, Form(choice): Form<RoleChoice>) -> Result<()> {
    require_permission(cx, Permission::UsersManage).await?;
    let id = *path_param::<UserId>(cx)?;
    let page = href!(super::user, UserId(id)).resolve(cx);
    match api::assign_role(cx, id, &choice.role).await {
        Ok(()) => done(page),
        Err(err) => Err(refused(cx, page, err)),
    }
}
