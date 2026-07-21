use serde::{Deserialize, Serialize};

use super::TokenKind;
use crate::identity::domain::UserId;

/// Claims carried by a signed confirmation token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationClaims {
	pub sub: i64,
	pub email: String,
	pub kind: TokenKind,
	pub new_email: Option<String>,
	pub exp: i64,
	pub iat: i64,
	/// Unique token id, so two tokens issued in the same second still differ.
	pub jti: String,
}

impl ConfirmationClaims {
	pub fn user_id(&self) -> UserId {
		UserId(self.sub)
	}
}
