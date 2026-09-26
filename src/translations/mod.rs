//! Translations bounded context: the ICU MessageFormat catalogue clients
//! render every message key with. Defaults ship with the binary and are
//! synchronized into the database at startup; an admin may override any
//! message, and a later reset brings the shipped default back.

pub mod application;
pub mod domain;
pub mod infrastructure;

pub use application::TranslationsService;
pub use domain::{Language, LocalizedText};
