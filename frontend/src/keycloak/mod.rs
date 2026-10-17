//! The way into Keycloak's pages and the person's account there: the sign-in
//! in the reader's theme and language, the account actions, the second factor
//! and the language kept on the account. The rest of the front-end uses only
//! what this module re-exports. The cookies the reader shares with Keycloak's
//! pages are `keycloak_cookies`, below the theme and the language this module
//! reads.

mod account;
mod signing_in;

pub use account::{AccountAction, SecondFactor, keep_account_language, second_factor};
pub use signing_in::{account_action, authorize, register, sign_in};
