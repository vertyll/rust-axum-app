pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_users_table;
mod m20220102_000001_create_roles_table;
mod m20220103_000001_create_user_roles_table;
mod m20220104_000001_create_refresh_tokens_table;
mod m20220105_000001_create_files_table;
mod m20220106_000001_create_users_email_history_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
	fn migrations() -> Vec<Box<dyn MigrationTrait>> {
		vec![
			Box::new(m20220101_000001_create_users_table::Migration),
			Box::new(m20220102_000001_create_roles_table::Migration),
			Box::new(m20220103_000001_create_user_roles_table::Migration),
			Box::new(m20220104_000001_create_refresh_tokens_table::Migration),
			Box::new(m20220105_000001_create_files_table::Migration),
			Box::new(m20220106_000001_create_users_email_history_table::Migration),
		]
	}
}
