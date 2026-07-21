use crate::identity::application::ports::PasswordHasher;
use crate::identity::domain::{IdentityError, PasswordHash};

/// Deterministic stand-in for Argon2: `hash(p) == "hashed:" + p`.
#[derive(Clone, Default)]
pub(crate) struct FakeHasher;

impl PasswordHasher for FakeHasher {
	async fn hash(&self, plaintext: String) -> Result<PasswordHash, IdentityError> {
		Ok(PasswordHash::new(format!("hashed:{plaintext}")))
	}

	async fn verify(&self, plaintext: String, hash: PasswordHash) -> Result<bool, IdentityError> {
		Ok(hash.as_str() == format!("hashed:{plaintext}"))
	}
}
