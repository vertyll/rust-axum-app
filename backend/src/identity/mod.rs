//! Identity bounded context: accounts mirrored from Keycloak, browser
//! sessions on Keycloak's hosted pages, and the auth guards other modules use.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::Caller;
pub use application::permissions::CATALOGUE;

pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "identity",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
pub use domain::permission::{Catalogue, Declared, Permission, StockRole};
pub use domain::role::RoleName;
pub use domain::user::UserId;
pub use infrastructure::http::{Auth, Permitted, require};
