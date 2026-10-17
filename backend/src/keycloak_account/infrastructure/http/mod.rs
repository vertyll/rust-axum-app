//! The signed-in person's Keycloak account over HTTP: `GET /me/second-factor`
//! and `PUT /me/language`, nested by the composition root under `/api/users`
//! behind the auth guard.

mod error;
mod routes;

pub use error::ApiError;
pub use routes::account_router;
