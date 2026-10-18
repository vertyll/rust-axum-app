//! What other modules' HTTP adapters may use of identity: the extractors that
//! require a signed-in caller, one whose roles grant a permission, or the
//! caller's access token. A domain or an application never takes them.

pub use super::infrastructure::http::{Auth, CallerToken, Permitted};
