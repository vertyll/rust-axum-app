//! Argon2 adapter for `PasswordHasher`: both operations run on the blocking
//! pool — Argon2 is deliberately expensive CPU work (the old code hashed
//! inline in handlers, stalling the runtime).
//! pool — Argon2 is deliberately expensive CPU work.
//!
//! Argon2 is deliberately expensive CPU work, so both operations run on the
//! blocking thread pool. The previous implementation hashed inline in the
//! request handlers, stalling the async runtime for tens of milliseconds
//! per login/registration.

use argon2::Argon2;
use argon2::password_hash::{
	PasswordHash as ParsedHash, PasswordHasher as _, PasswordVerifier as _, SaltString, rand_core::OsRng,
};

use crate::identity::application::ports::PasswordHasher;
use crate::identity::domain::{IdentityError, PasswordHash};

#[derive(Clone, Default)]
pub struct Argon2PasswordHasher;

impl PasswordHasher for Argon2PasswordHasher {
	async fn hash(&self, plaintext: String) -> Result<PasswordHash, IdentityError> {
		tokio::task::spawn_blocking(move || {
			let salt = SaltString::generate(&mut OsRng);
			Argon2::default()
				.hash_password(plaintext.as_bytes(), &salt)
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
