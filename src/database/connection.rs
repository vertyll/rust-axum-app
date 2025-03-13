use crate::config::database_config::DatabaseConfig;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
	let pool = PgPoolOptions::new()
		.max_connections(config.max_connections)
		.min_connections(config.min_connections.unwrap_or(1))
		.connect(&config.connection_string())
		.await?;

	Ok(pool)
}
