//! Proof Key for Code Exchange (RFC 7636) and the sign-in state.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CHALLENGE_METHOD: &str = "S256";

const STATE_BYTES: usize = 32;
const VERIFIER_BYTES: usize = 64;

/// What `/auth/authorize` remembers for the callback, in the browser's session.
#[derive(Clone, Serialize, Deserialize)]
pub struct SignInTransaction {
	pub state: String,
	pub code_verifier: String,
}

impl SignInTransaction {
	pub fn new() -> Self {
		Self {
			state: random(STATE_BYTES),
			code_verifier: random(VERIFIER_BYTES),
		}
	}
}

impl Default for SignInTransaction {
	fn default() -> Self {
		Self::new()
	}
}

pub fn challenge_of(code_verifier: &str) -> String {
	URL_SAFE_NO_PAD.encode(Sha256::digest(code_verifier.as_bytes()))
}

/// Compares in time independent of where the values first differ.
pub fn same_state(expected: &str, received: &str) -> bool {
	expected.len() == received.len()
		&& expected
			.bytes()
			.zip(received.bytes())
			.fold(0u8, |difference, (a, b)| difference | (a ^ b))
			== 0
}

fn random(bytes: usize) -> String {
	let mut buffer = vec![0u8; bytes];
	rand::rng().fill(&mut buffer[..]);
	URL_SAFE_NO_PAD.encode(buffer)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn challenge_matches_the_rfc_7636_example() {
		assert_eq!(
			challenge_of("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
			"E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
		);
	}

	#[test]
	fn every_transaction_is_new() {
		assert_ne!(SignInTransaction::new().state, SignInTransaction::new().state);
	}

	#[test]
	fn states_compare_exactly() {
		assert!(same_state("abc", "abc"));
		assert!(!same_state("abc", "abd"));
		assert!(!same_state("abc", "abcd"));
	}
}
