use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
	pub host: String,
	pub port: u16,
	pub username: String,
	pub password: String,
	pub db_name: String,
	pub max_connections: u32,
	pub min_connections: Option<u32>,
}

impl DatabaseConfig {
	pub fn connection_string(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}/{}",
			self.username, self.password, self.host, self.port, self.db_name
		)
	}
}
