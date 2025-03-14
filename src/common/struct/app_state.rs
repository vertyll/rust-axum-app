use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct AppState {
	pub db: DatabaseConnection,
	pub jwt_access_token_secret: String,
}

impl AppState {
	pub fn new(db: DatabaseConnection, jwt_access_token_secret: String) -> Self {
		Self {
			db,
			jwt_access_token_secret,
		}
	}
}
