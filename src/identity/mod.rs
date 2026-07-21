//! Identity bounded context.

pub mod application;
pub mod domain;
pub mod infrastructure;

// Public API for other modules (auth guards + claims).
pub use application::token::AccessClaims;
pub use domain::role::RoleName;
pub use domain::user::UserId;
pub use infrastructure::http::{Auth, RequireAdmin};
