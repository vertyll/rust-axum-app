//! Translations bounded context: the ICU `MessageFormat` catalogue clients
//! render every message key with. Defaults ship with the binary and are
//! synchronized into the database at startup; an admin may override any
//! message, and a later reset brings the shipped default back.

pub mod application;
pub mod contracts;
pub mod domain;
pub mod infrastructure;

pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "translations",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
