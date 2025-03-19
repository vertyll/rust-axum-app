use crate::common::error::app_error::AppError;
use crate::di::DatabaseConnectionTrait;
use crate::files::dto::create_file_dto::CreateFileDto;
use crate::files::dto::update_file_dto::UpdateFileDto;
use crate::files::entities::files::{self, ActiveModel as FileActiveModel, Entity as File, Model as FileModel};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone)]
pub struct FilesRepository {
	pub database_connection: Arc<dyn DatabaseConnectionTrait>,
}

impl FilesRepository {
	pub fn new(database_connection: Arc<dyn DatabaseConnectionTrait>) -> Self {
		Self { database_connection }
	}
}

#[async_trait]
pub trait FilesRepositoryTrait: Send + Sync {
	fn get_db(&self) -> &DatabaseConnection;
	async fn find_all(&self) -> Result<Vec<FileModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<FileModel, AppError>;
	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateFileDto,
		filename: String,
		path: String,
		url: String,
		metadata: Value,
	) -> Result<FileModel, AppError>;
	async fn update(&self, id: i32, dto: UpdateFileDto) -> Result<FileModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
	async fn soft_delete(&self, id: i32, user_id: i32) -> Result<(), AppError>;
}

#[async_trait]
impl FilesRepositoryTrait for FilesRepository {
	fn get_db(&self) -> &DatabaseConnection {
		self.database_connection.get_connection()
	}

	async fn find_all(&self) -> Result<Vec<FileModel>, AppError> {
		let files = File::find()
			.filter(files::Column::DeletedAt.is_null())
			.all(self.get_db())
			.await?;

		Ok(files)
	}

	async fn find_by_id(&self, id: i32) -> Result<FileModel, AppError> {
		let file = File::find_by_id(id)
			.filter(files::Column::DeletedAt.is_null())
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(file)
	}

	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateFileDto,
		filename: String,
		path: String,
		url: String,
		metadata: Value,
	) -> Result<FileModel, AppError> {
		let now = chrono::Utc::now();
		let storage_type = dto.storage_type.unwrap_or_else(|| "local".to_string());

		let file_active_model = FileActiveModel {
			filename: Set(filename),
			original_name: Set(dto.original_name),
			path: Set(path),
			mime_type: Set(dto.mime_type),
			encoding: Set(dto.encoding),
			size: Set(dto.size),
			storage_type: Set(storage_type),
			url: Set(url),
			metadata: Set(metadata),
			created_at: Set(now.into()),
			updated_at: Set(Some(now.into())),
			deleted_at: Set(None),
			deleted_by_user_id: Set(None),
			..Default::default()
		};

		let file = file_active_model.insert(transaction).await?;

		Ok(file)
	}

	async fn update(&self, id: i32, dto: UpdateFileDto) -> Result<FileModel, AppError> {
		let file = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut file_active_model: FileActiveModel = file.into();

		if let Some(original_name) = dto.original_name {
			file_active_model.original_name = Set(original_name);
		}

		if let Some(mime_type) = dto.mime_type {
			file_active_model.mime_type = Set(mime_type);
		}

		if let Some(encoding) = dto.encoding {
			file_active_model.encoding = Set(encoding);
		}

		if let Some(size) = dto.size {
			file_active_model.size = Set(size);
		}

		if let Some(storage_type) = dto.storage_type {
			file_active_model.storage_type = Set(storage_type);
		}

		if let Some(url) = dto.url {
			file_active_model.url = Set(url);
		}

		file_active_model.updated_at = Set(Some(now.into()));

		let updated_file = file_active_model.update(self.get_db()).await?;

		Ok(updated_file)
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		let file = self.find_by_id(id).await?;
		let file_active_model: FileActiveModel = file.into();

		file_active_model.delete(self.get_db()).await?;

		Ok(())
	}

	async fn soft_delete(&self, id: i32, user_id: i32) -> Result<(), AppError> {
		let file = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut file_active_model: FileActiveModel = file.into();
		file_active_model.deleted_at = Set(Some(now.into()));
		file_active_model.deleted_by_user_id = Set(Some(user_id));

		file_active_model.update(self.get_db()).await?;

		Ok(())
	}
}
