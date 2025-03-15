mod roles_seeder;

use crate::common::error::app_error::AppError;
use sea_orm::DatabaseConnection;

pub async fn run_seeders(db: &DatabaseConnection) -> Result<(), AppError> {
	println!("Running seeders...");

	roles_seeder::seed_roles(db).await?;

	println!("Seeders completed successfully.");
	Ok(())
}
