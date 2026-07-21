use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AccessTokenResponse {
	pub access_token: String,
}
