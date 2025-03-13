use sqlx::{PgPool, Postgres, migrate::MigrateDatabase};
use std::path::Path;

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
	let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

	if !Postgres::database_exists(&db_url).await? {
		tracing::info!("Creating database...");
		Postgres::create_database(&db_url).await?;
	}

	let migrations_path = Path::new("./migrations");

	tracing::info!("Run migrations with {:?}...", migrations_path);

	sqlx::migrate::Migrator::new(migrations_path).await?.run(pool).await?;

	tracing::info!("Migration completed successfully");

	Ok(())
}
