use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(UsersEmailHistory::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(UsersEmailHistory::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(UsersEmailHistory::UserId).integer().not_null())
					.col(ColumnDef::new(UsersEmailHistory::OldEmail).string().not_null())
					.col(ColumnDef::new(UsersEmailHistory::NewEmail).string().not_null())
					.col(
						ColumnDef::new(UsersEmailHistory::EmailChangeAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(
						ColumnDef::new(UsersEmailHistory::CreatedAt)
							.timestamp_with_time_zone()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.col(
						ColumnDef::new(UsersEmailHistory::UpdatedAt)
							.timestamp_with_time_zone()
							.null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_users_email_history_user")
							.from(UsersEmailHistory::Table, UsersEmailHistory::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade)
							.on_update(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(UsersEmailHistory::Table).to_owned())
			.await
	}
}

#[derive(DeriveIden)]
enum UsersEmailHistory {
	Table,
	Id,
	UserId,
	OldEmail,
	NewEmail,
	EmailChangeAt,
	CreatedAt,
	UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
}
