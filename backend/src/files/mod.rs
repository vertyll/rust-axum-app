//! Files bounded context: upload, storage and metadata of files.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "files",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
