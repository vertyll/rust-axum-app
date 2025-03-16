#[derive(Clone)]
pub struct TokenState {
	pub jwt_access_token_secret: String,
	pub jwt_access_token_expires_in: i64,
	pub jwt_refresh_token_secret: String,
	pub jwt_refresh_token_expires_in: i64,
}

impl TokenState {
	pub fn new(
		jwt_access_token_secret: String,
		jwt_access_token_expires_in: i64,
		jwt_refresh_token_secret: String,
		jwt_refresh_token_expires_in: i64,
	) -> Self {
		Self {
			jwt_access_token_secret,
			jwt_access_token_expires_in,
			jwt_refresh_token_secret,
			jwt_refresh_token_expires_in,
		}
	}
}
