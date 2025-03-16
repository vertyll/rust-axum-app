use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Users::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Users::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Users::Username).string().not_null().unique_key())
					.col(ColumnDef::new(Users::Email).string().not_null().unique_key())
					.col(ColumnDef::new(Users::PasswordHash).string().not_null())
					.col(
						ColumnDef::new(Users::IsEmailConfirmed)
							.boolean()
							.not_null()
							.default(false),
					)
					.col(ColumnDef::new(Users::EmailConfirmationToken).string())
					.col(ColumnDef::new(Users::EmailConfirmationTokenExpiry).timestamp_with_time_zone())
					.col(ColumnDef::new(Users::EmailChangeToken).string())
					.col(ColumnDef::new(Users::EmailChangeTokenExpiry).timestamp_with_time_zone())
					.col(ColumnDef::new(Users::PasswordResetToken).string())
					.col(ColumnDef::new(Users::PasswordResetTokenExpiry).timestamp_with_time_zone())
					.col(ColumnDef::new(Users::PendingEmail).string())
					.col(
						ColumnDef::new(Users::CreatedAt)
							.timestamp_with_time_zone()
							.not_null()
							.default(Expr::current_timestamp()),
					)
					.col(ColumnDef::new(Users::UpdatedAt).timestamp_with_time_zone().null())
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table(Users::Table).to_owned()).await
	}
}

#[derive(DeriveIden)]
enum Users {
	Table,
	Id,
	Username,
	Email,
	PasswordHash,
	IsEmailConfirmed,
	EmailConfirmationToken,
	EmailConfirmationTokenExpiry,
	EmailChangeToken,
	EmailChangeTokenExpiry,
	PasswordResetToken,
	PasswordResetTokenExpiry,
	PendingEmail,
	CreatedAt,
	UpdatedAt,
}
