//! The way into Keycloak's pages and the person's account there. The rest of
//! the front-end uses only what this module re-exports. Its two parts depend
//! on different things and not on each other: what is shared with Keycloak's
//! pages, which the theme and the language read, and the account, which goes
//! through the back-end and its proxy.

mod account;
mod shared;

pub use account::{AccountAction, account_action, keep_account_language, second_factor};
pub use shared::{Preference, chosen, share};
