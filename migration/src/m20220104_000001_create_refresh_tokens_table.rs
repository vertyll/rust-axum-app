use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(RefreshTokens::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(RefreshTokens::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(RefreshTokens::Token).string().not_null())
					.col(ColumnDef::new(RefreshTokens::UserId).integer().not_null())
					.col(
						ColumnDef::new(RefreshTokens::ExpiresAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(
						ColumnDef::new(RefreshTokens::CreatedAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(
						ColumnDef::new(RefreshTokens::UpdatedAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_refresh_tokens_user_id")
							.from(RefreshTokens::Table, RefreshTokens::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(RefreshTokens::Table).to_owned())
			.await
	}
}

#[derive(Iden)]
enum RefreshTokens {
	Table,
	Id,
	UserId,
	Token,
	ExpiresAt,
	CreatedAt,
	UpdatedAt,
}

#[derive(Iden)]
enum Users {
	Table,
	Id,
}
