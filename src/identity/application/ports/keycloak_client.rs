use crate::identity::application::session::TokenPair;
use crate::identity::domain::IdentityError;

/// Keycloak's token endpoint, as the sign-in flow needs it.
pub trait KeycloakClient: Clone + Send + Sync + 'static {
	/// Redeems an authorization code; `SignInRejected` when Keycloak refuses it.
	fn exchange(
		&self,
		code: &str,
		code_verifier: &str,
	) -> impl Future<Output = Result<TokenPair, IdentityError>> + Send;

	/// Rotates a refresh token; `SessionExpired` when Keycloak refuses it.
	/// Concurrent calls with the same token share one request.
	fn refresh(&self, refresh_token: &str) -> impl Future<Output = Result<TokenPair, IdentityError>> + Send;

	/// Ends the Keycloak session behind a refresh token, best-effort.
	fn revoke(&self, refresh_token: &str) -> impl Future<Output = ()> + Send;
}
