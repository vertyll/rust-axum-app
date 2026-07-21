use serde::{Deserialize, Serialize};

/// What a single-use confirmation token is allowed to confirm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenKind {
	EmailConfirmation,
	EmailChange,
	PasswordReset,
}
