use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::content::Form;
use topcoat::router::{href, path_param, route};

use super::super::super::api;
use super::super::UserId;
use super::RoleChoice;
use crate::auth::{Permission, require_permission};
use crate::forms::{done, refused};

#[route(POST)]
pub(crate) async fn remove_role(cx: &Cx, Form(choice): Form<RoleChoice>) -> Result<()> {
    require_permission(cx, Permission::UsersManage).await?;
    let id = *path_param::<UserId>(cx)?;
    let page = href!(super::super::user, UserId(id)).resolve(cx);
    match api::remove_role(cx, id, &choice.role).await {
        Ok(()) => done(page),
        Err(err) => Err(refused(cx, page, err)),
    }
}
