use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(UserRoles::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(UserRoles::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(UserRoles::UserId).integer().not_null())
					.col(ColumnDef::new(UserRoles::RoleId).integer().not_null())
					.col(
						ColumnDef::new(UserRoles::CreatedAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.col(
						ColumnDef::new(UserRoles::UpdatedAt)
							.timestamp_with_time_zone()
							.not_null(),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_roles_user_id")
							.from(UserRoles::Table, UserRoles::UserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.foreign_key(
						ForeignKey::create()
							.name("fk_user_roles_role_id")
							.from(UserRoles::Table, UserRoles::RoleId)
							.to(Roles::Table, Roles::Id)
							.on_delete(ForeignKeyAction::Cascade),
					)
					.to_owned(),
			)
			.await?;

		manager
			.create_index(
				Index::create()
					.name("idx_user_role_unique")
					.table(UserRoles::Table)
					.col(UserRoles::UserId)
					.col(UserRoles::RoleId)
					.unique()
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(UserRoles::Table).to_owned())
			.await
	}
}

#[derive(Iden)]
enum UserRoles {
	Table,
	Id,
	UserId,
	RoleId,
	CreatedAt,
	UpdatedAt,
}

#[derive(Iden)]
enum Users {
	Table,
	Id,
}

#[derive(Iden)]
enum Roles {
	Table,
	Id,
}
