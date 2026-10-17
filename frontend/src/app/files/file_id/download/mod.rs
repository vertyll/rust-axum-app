use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{path_param, route};

use super::super::api;
use super::FileId;
use crate::auth::{Permission, require_permission};
use crate::backend::BackendError;
use crate::forms::done;

/// Sends the reader to a link to the file's bytes, asked for now: a link kept in
/// the page would expire while the page is open.
#[route(GET)]
pub(crate) async fn download_file(cx: &Cx) -> Result<()> {
    require_permission(cx, Permission::FilesView).await?;
    let id = *path_param::<FileId>(cx)?;
    let link = api::download_link(cx, id)
        .await
        .map_err(BackendError::into_page_error)?;
    done(link.url)
}
