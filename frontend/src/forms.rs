//! How a form answers. A change posts to a route, which answers `303 See
//! Other` to a page when the back-end accepts it, so a reload does not repeat
//! it. When the back-end refuses, the route renders the form's page again as
//! a `GET`, carrying the refusal and what the reader typed, so nothing they
//! entered is lost.

use std::collections::BTreeMap;

use topcoat::context::{Cx, try_request_context};
use topcoat::router::error::{internal_server_error, rewrite, see_other};
use topcoat::router::header::{CONTENT_LENGTH, CONTENT_TYPE};
use topcoat::router::request::headers;
use topcoat::router::{Body, Method, StatusCode, href};

use crate::backend::{BackendError, Problem};

#[derive(Debug, Clone)]
pub struct FormFailure {
    pub problem: Problem,
    pub values: BTreeMap<String, String>,
}

impl FormFailure {
    #[must_use]
    pub fn value(&self, field: &str) -> Option<&str> {
        self.values.get(field).map(String::as_str)
    }
}

pub fn form_failure(cx: &Cx) -> Option<&FormFailure> {
    try_request_context::<FormFailure>(cx)
}

pub fn typed_or(cx: &Cx, field: &str, current: &str) -> String {
    form_failure(cx)
        .and_then(|failure| failure.value(field))
        .unwrap_or(current)
        .to_owned()
}

pub fn done(location: impl AsRef<str>) -> topcoat::Result<()> {
    Err(see_other(location).into())
}

pub fn refused(cx: &Cx, page: impl AsRef<str>, error: BackendError) -> topcoat::Error {
    refused_with(cx, page, error, Vec::<(String, String)>::new())
}

/// The answer to a form the back-end did not accept. A refusal is shown on
/// the page at `page`, with the `values` the reader typed; an ended session
/// sends the reader home to sign in again, and anything else is a server
/// error.
pub fn refused_with<K: Into<String>>(
    cx: &Cx,
    page: impl AsRef<str>,
    error: BackendError,
    values: impl IntoIterator<Item = (K, String)>,
) -> topcoat::Error {
    match error {
        BackendError::Refused(problem) if problem.status == StatusCode::UNAUTHORIZED => {
            see_other(href!(crate::app::home).resolve(cx)).into()
        }
        BackendError::Refused(problem) => {
            let mut get_headers = headers(cx).clone();
            get_headers.remove(CONTENT_TYPE);
            get_headers.remove(CONTENT_LENGTH);
            let failure = FormFailure {
                problem,
                values: values.into_iter().map(|(name, value)| (name.into(), value)).collect(),
            };
            rewrite(page.as_ref(), Body::empty())
                .method(Method::GET)
                .headers(get_headers)
                .with(failure)
                .into()
        }
        other => internal_server_error(other).into(),
    }
}
