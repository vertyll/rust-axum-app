/// The pair returned by `register` and `login`.
#[derive(Debug, Clone)]
pub struct AuthTokens {
	pub access_token: String,
	pub refresh_token: String,
}
