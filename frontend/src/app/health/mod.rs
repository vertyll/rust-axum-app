use topcoat::Result;
use topcoat::router::route;

/// Answers while the server runs, for the container's health check; it does
/// not ask the back-end, which has a check of its own.
#[route(GET)]
pub(crate) async fn health() -> Result<&'static str> {
    Ok("UP")
}
