//! Keycloak account bounded context: what a signed-in person keeps on their
//! Keycloak account and changes on Keycloak's own pages, the second factor and
//! the language those pages show them in. It is read and written with the
//! person's own access token, through Keycloak's account API, so it needs no
//! right over anyone else's account. It uses the auth guards of `identity`, as
//! the other modules do, and `identity` never imports it.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "keycloak_account",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
