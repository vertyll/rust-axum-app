use crate::identity::application::session::VerifiedToken;
use crate::identity::domain::IdentityError;

/// Checks an access token's signature, issuer, audience and expiry.
pub trait TokenVerifier: Clone + Send + Sync + 'static {
	fn verify(&self, token: &str) -> impl Future<Output = Result<VerifiedToken, IdentityError>> + Send;
}
