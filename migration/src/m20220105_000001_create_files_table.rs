use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.create_table(
				Table::create()
					.table(Files::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(Files::Id)
							.integer()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(Files::Filename).string().not_null())
					.col(ColumnDef::new(Files::OriginalName).string().not_null())
					.col(ColumnDef::new(Files::Path).string().not_null())
					.col(ColumnDef::new(Files::MimeType).string().not_null())
					.col(ColumnDef::new(Files::Encoding).string().not_null())
					.col(ColumnDef::new(Files::Size).integer().not_null())
					.col(ColumnDef::new(Files::StorageType).string().not_null())
					.col(ColumnDef::new(Files::Url).string().not_null())
					.col(ColumnDef::new(Files::Metadata).json().not_null())
					.col(ColumnDef::new(Files::CreatedAt).timestamp_with_time_zone().not_null())
					.col(ColumnDef::new(Files::UpdatedAt).timestamp_with_time_zone().null())
					.col(ColumnDef::new(Files::DeletedAt).timestamp_with_time_zone().null())
					.col(ColumnDef::new(Files::DeletedByUserId).integer().null())
					.foreign_key(
						ForeignKey::create()
							.name("fk_files_deleted_by_user_id")
							.from(Files::Table, Files::DeletedByUserId)
							.to(Users::Table, Users::Id)
							.on_delete(ForeignKeyAction::SetNull),
					)
					.to_owned(),
			)
			.await
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager.drop_table(Table::drop().table(Files::Table).to_owned()).await
	}
}

#[derive(Iden)]
enum Files {
	Table,
	Id,
	Filename,
	OriginalName,
	Path,
	MimeType,
	Encoding,
	Size,
	StorageType,
	Url,
	Metadata,
	CreatedAt,
	UpdatedAt,
	DeletedAt,
	DeletedByUserId,
}

#[derive(Iden)]
enum Users {
	Table,
	Id,
}
