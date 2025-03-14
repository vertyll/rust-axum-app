use crate::config::database_config::DatabaseConfig;
use sea_orm::{Database, DatabaseConnection};

pub async fn connect(config: &DatabaseConfig) -> Result<DatabaseConnection, sea_orm::DbErr> {
	let connection_string = config.connection_string();

	let db = Database::connect(connection_string).await?;

	Ok(db)
}
