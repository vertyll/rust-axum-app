use topcoat::Result;
use topcoat::context::Cx;
use topcoat::router::error::internal_server_error;
use topcoat::router::{href, route};

use crate::auth::api;
use crate::forms::done;

/// Ends the session at the back-end, which also ends it at Keycloak, and
/// passes on the back-end's removal of the session cookie.
#[route(POST)]
pub(crate) async fn sign_out(cx: &Cx) -> Result<()> {
    api::sign_out(cx).await.map_err(internal_server_error)?;
    done(href!(super::home).resolve(cx))
}
