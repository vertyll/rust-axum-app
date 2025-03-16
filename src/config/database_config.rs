use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
	pub db_host: String,
	pub db_port: u16,
	pub db_username: String,
	pub db_password: String,
	pub db_name: String,
	pub db_max_connections: u32,
	pub db_min_connections: Option<u32>,
}

impl DatabaseConfig {
	pub fn connection_string(&self) -> String {
		format!(
			"postgres://{}:{}@{}:{}/{}",
			self.db_username, self.db_password, self.db_host, self.db_port, self.db_name
		)
	}
}
