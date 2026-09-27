//! Argon2 adapter for `PasswordHasher`. Argon2 is deliberately expensive CPU
//! work, so both operations run on the blocking thread pool.

use argon2::Argon2;
use argon2::password_hash::{PasswordHasher as _, PasswordVerifier as _, phc::PasswordHash as ParsedHash};

use crate::identity::application::ports::PasswordHasher;
use crate::identity::domain::{IdentityError, PasswordHash};

#[derive(Clone, Default)]
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
	async fn hash(&self, plaintext: String) -> Result<PasswordHash, IdentityError> {
		tokio::task::spawn_blocking(move || {
			Argon2::default()
				.hash_password(plaintext.as_bytes())
				.map(|hash| PasswordHash::new(hash.to_string()))
				.map_err(|err| {
					tracing::error!("password hashing failed: {err}");
					IdentityError::HashingFailure
				})
		})
		.await
		.map_err(|_| IdentityError::HashingFailure)?
	}

	async fn verify(&self, plaintext: String, hash: PasswordHash) -> Result<bool, IdentityError> {
		tokio::task::spawn_blocking(move || {
			let parsed = ParsedHash::new(hash.as_str()).map_err(|err| {
				tracing::error!("stored password hash is unparsable: {err}");
				IdentityError::HashingFailure
			})?;
			Ok(Argon2::default().verify_password(plaintext.as_bytes(), &parsed).is_ok())
		})
		.await
		.map_err(|_| IdentityError::HashingFailure)?
	}
}
#[cfg(test)]
mod tests {
	use super::Argon2PasswordHasher;
	use crate::identity::application::ports::PasswordHasher as _;

	#[tokio::test]
	async fn hash_verify_round_trip() {
		let hasher = Argon2PasswordHasher;
		let hash = hasher.hash("correct horse".into()).await.unwrap();
		assert!(hasher.verify("correct horse".into(), hash.clone()).await.unwrap());
		assert!(!hasher.verify("battery staple".into(), hash).await.unwrap());
	}
}
