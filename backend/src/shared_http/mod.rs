//! What every module's HTTP adapter shares: problem documents and paging.

pub mod pagination;

/// The messages this module answers with.
pub const MESSAGES: crate::shared_kernel::messages::Messages = crate::shared_kernel::messages::Messages {
    owner: "shared_http",
    pl: include_str!("messages/pl.json"),
    en: include_str!("messages/en.json"),
};
pub mod problem;
