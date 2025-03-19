use crate::common::enums::file_storage_type_enum::FileStorageTypeEnum;
use crate::common::error::app_error::AppError;
use crate::config::app_config::AppConfig;
use crate::files::dto::create_file_dto::CreateFileDto;
use crate::files::dto::update_file_dto::UpdateFileDto;
use crate::files::entities::files::Model as FileModel;
use crate::files::repositories::files_repository::{FilesRepository, FilesRepositoryTrait};
use crate::files::strategies::storage_strategy::{StorageStrategy, get_storage_strategy};
use crate::i18n::setup::translate;
use async_trait::async_trait;
use axum::extract::Multipart;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::sync::Arc;
use crate::di::AppConfigTrait;

#[derive(Clone)]
pub struct FilesService {
	pub files_repository: Arc<dyn FilesRepositoryTrait>,
	pub app_config: Arc<dyn AppConfigTrait>,
}

impl FilesService {
	pub fn new(files_repository: Arc<dyn FilesRepositoryTrait>, app_config: Arc<dyn AppConfigTrait>) -> Self {
		Self {
			files_repository,
			app_config,
		}
	}
}

#[async_trait]
pub trait FilesServiceTrait: Send + Sync {
	async fn begin_transaction(&self) -> Result<DatabaseTransaction, AppError>;
	async fn find_all(&self) -> Result<Vec<FileModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<FileModel, AppError>;
	async fn upload(&self, multipart: Multipart, storage_type: Option<String>) -> Result<FileModel, AppError>;
	async fn update(&self, id: i32, dto: UpdateFileDto) -> Result<FileModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
	async fn soft_delete(&self, id: i32, user_id: i32) -> Result<(), AppError>;
}

#[async_trait]
impl FilesServiceTrait for FilesService {
	async fn begin_transaction(&self) -> Result<DatabaseTransaction, AppError> {
		Ok(self.files_repository.get_db().begin().await?)
	}

	async fn find_all(&self) -> Result<Vec<FileModel>, AppError> {
		self.files_repository.find_all().await
	}

	async fn find_by_id(&self, id: i32) -> Result<FileModel, AppError> {
		self.files_repository.find_by_id(id).await
	}

	async fn upload(&self, mut multipart: Multipart, storage_type: Option<String>) -> Result<FileModel, AppError> {
		let storage_type = storage_type.unwrap_or_else(|| FileStorageTypeEnum::Local.to_string());
		let storage_strategy = get_storage_strategy(&storage_type, self.app_config.as_ref());

		let mut file_data = Vec::new();
		let mut original_name = String::new();
		let mut mime_type = String::new();
		let mut encoding = String::new();
		let mut size = 0;

		while let Some(field) = multipart.next_field().await.map_err(|e| {
			tracing::error!("Error reading multipart field: {}", e);
			AppError::BadRequest(translate("files.errors.upload"))
		})? {
			let name = field.name().unwrap_or("").to_string();

			if name == "file" {
				original_name = field.file_name().unwrap_or("unknown").to_string();
				mime_type = field.content_type().unwrap_or("application/octet-stream").to_string();
				encoding = "base64".to_string();

				file_data = field
					.bytes()
					.await
					.map_err(|e| {
						tracing::error!("Error reading file data: {}", e);
						AppError::BadRequest(translate("files.errors.upload"))
					})?
					.to_vec();

				size = file_data.len() as i32;
			}
		}

		if file_data.is_empty() {
			return Err(AppError::BadRequest(translate("files.errors.no_file")));
		}

		let file_info = storage_strategy
			.save_file(file_data, &original_name, &mime_type)
			.await?;

		let db = self.files_repository.get_db();
		let transaction = db.begin().await?;

		let create_file_dto = CreateFileDto {
			original_name: file_info.original_name,
			mime_type,
			encoding,
			size,
			storage_type: Some(storage_type),
		};

		let result = self
			.files_repository
			.create_in_transaction(
				&transaction,
				create_file_dto,
				file_info.filename,
				file_info.path.clone(),
				file_info.url,
				file_info.metadata,
			)
			.await;

		match result {
			Ok(file) => {
				transaction.commit().await?;
				Ok(file)
			}
			Err(e) => {
				transaction.rollback().await?;
				let _ = storage_strategy.delete_file(&file_info.path).await;
				Err(e)
			}
		}
	}

	async fn update(&self, id: i32, dto: UpdateFileDto) -> Result<FileModel, AppError> {
		self.files_repository.update(id, dto).await
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		let file = self.files_repository.find_by_id(id).await?;

		let storage_strategy = get_storage_strategy(&file.storage_type, self.app_config.as_ref());
		storage_strategy.delete_file(&file.path).await?;

		self.files_repository.delete(id).await
	}

	async fn soft_delete(&self, id: i32, user_id: i32) -> Result<(), AppError> {
		self.files_repository.soft_delete(id, user_id).await
	}
}
