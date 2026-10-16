use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, path_param, route};

use super::super::api;
use super::FileId;
use crate::auth::{Permission, require_permission};
use crate::forms::{done, refused};

#[route(POST)]
pub(crate) async fn delete_file(cx: &Cx) -> Result<()> {
    require_permission(cx, Permission::FilesManage).await?;
    let id = *path_param::<FileId>(cx)?;
    match api::delete(cx, id).await {
        Ok(()) => done(href!(super::super::files).resolve(cx)),
        Err(err) => Err(refused(cx, href!(super::file, FileId(id)).resolve(cx), err)),
    }
}
