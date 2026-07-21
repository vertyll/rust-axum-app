use crate::identity::domain::{IdentityError, PasswordHash};

/// Password hashing — async so the adapter can run Argon2 on the blocking
/// pool; owned args move into `spawn_blocking`.
pub trait PasswordHasher: Clone + Send + Sync + 'static {
	fn hash(&self, plaintext: String) -> impl Future<Output = Result<PasswordHash, IdentityError>> + Send;

	fn verify(&self, plaintext: String, hash: PasswordHash)
	-> impl Future<Output = Result<bool, IdentityError>> + Send;
}
