use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, path_param, route};

use super::super::api;
use super::RoleName;
use crate::auth::{Permission, require_permission};
use crate::forms::{done, refused};

#[route(POST)]
pub(crate) async fn delete_role(cx: &Cx) -> Result<()> {
    require_permission(cx, Permission::RolesManage).await?;
    let name = path_param::<RoleName>(cx).to_owned();
    match api::delete_role(cx, &name).await {
        Ok(()) => done(href!(super::super::roles).resolve(cx)),
        Err(err) => Err(refused(cx, href!(super::role, RoleName(name)).resolve(cx), err)),
    }
}
