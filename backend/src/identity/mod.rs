//! Identity bounded context: accounts mirrored from Keycloak, browser
//! sessions on Keycloak's hosted pages, and the auth guards other modules use.

pub mod application;
pub mod contracts;
pub mod domain;
pub mod guards;
pub mod infrastructure;

pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "identity",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
