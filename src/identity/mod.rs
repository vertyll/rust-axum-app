//! Identity bounded context: accounts mirrored from Keycloak, browser
//! sessions on Keycloak's hosted pages, and the auth guards other modules use.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::Caller;
pub use application::permissions::CATALOGUE;
pub use domain::permission::{Catalogue, Declared, Permission, StockRole};
pub use domain::role::RoleName;
pub use domain::user::UserId;
pub use infrastructure::http::{Auth, Permitted};
