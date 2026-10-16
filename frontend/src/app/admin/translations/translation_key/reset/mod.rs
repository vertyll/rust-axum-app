use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::{href, path_param, route};

use super::super::api as catalog;
use super::TranslationKey;
use crate::auth::{Permission, require_permission};
use crate::forms::{done, refused};

#[route(POST)]
pub(crate) async fn reset_translation(cx: &Cx) -> Result<()> {
    require_permission(cx, Permission::TranslationsEdit).await?;
    let key = path_param::<TranslationKey>(cx).to_owned();
    let page = href!(super::translation, TranslationKey(key.clone())).resolve(cx);
    match catalog::reset(cx, &key).await {
        Ok(_) => done(page),
        Err(err) => Err(refused(cx, page, err)),
    }
}
