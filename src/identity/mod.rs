//! Identity bounded context. Other modules use only this curated surface —
//! chiefly the auth guards and [`AccessClaims`]; the composition root also
//! wires the concrete adapters into the service.
//! e-mail confirmation and password flows.
//!
//! Other modules interact with identity only through this curated surface —
//! primarily the auth guards ([`Auth`], [`RequireAdmin`]) and
//! [`AccessClaims`]. The composition root additionally uses the application
//! service and the concrete adapters to wire everything together.

pub mod application;
pub mod domain;
pub mod infrastructure;

// Public API for other modules (auth guards + claims).
pub use application::token::AccessClaims;
pub use domain::role::RoleName;
pub use domain::user::UserId;
pub use infrastructure::http::{Auth, RequireAdmin};
