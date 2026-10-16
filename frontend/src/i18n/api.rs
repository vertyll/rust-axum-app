use std::collections::BTreeMap;

use reqwest::Method;
use topcoat::context::Cx;

use super::Lang;
use crate::backend::{self, BackendError};

pub async fn catalog(cx: &Cx, lang: Lang) -> Result<BTreeMap<String, String>, BackendError> {
    let path = format!("/api/translations/{}", lang.code());
    backend::json(cx, backend::public_request(cx, Method::GET, &path)).await
}
