use crate::common::error::app_error::AppError;
use crate::config::app_config::AppConfig;
use crate::di::IAppConfig;
use async_trait::async_trait;
use axum::extract::Multipart;
use serde_json::Value;
use shaku::{Component, Interface};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tokio::fs as tokio_fs;
use uuid::Uuid;

pub struct FileInfo {
	pub filename: String,
	pub original_name: String,
	pub path: String,
	pub url: String,
	pub metadata: Value,
}

#[async_trait]
pub trait IStorageStrategy: Interface {
	async fn save_file(&self, file_data: Vec<u8>, original_name: &str, mime_type: &str) -> Result<FileInfo, AppError>;
	async fn delete_file(&self, file_path: &str) -> Result<(), AppError>;
}

#[derive(Component)]
#[shaku(interface = IStorageStrategy)]
pub struct LocalStorageStrategyImpl {
	upload_dir: String,
	base_url: String,
}

impl LocalStorageStrategyImpl {
	pub fn new(upload_dir: String, base_url: String) -> Self {
		Self { upload_dir, base_url }
	}
}

#[async_trait]
impl IStorageStrategy for LocalStorageStrategyImpl {
	async fn save_file(&self, file_data: Vec<u8>, original_name: &str, _mime_type: &str) -> Result<FileInfo, AppError> {
		if !Path::new(&self.upload_dir).exists() {
			fs::create_dir_all(&self.upload_dir).map_err(|e| {
				tracing::error!("Failed to create upload directory: {}", e);
				AppError::InternalError
			})?;
		}

		let file_extension = Path::new(original_name)
			.extension()
			.and_then(|ext| ext.to_str())
			.unwrap_or("");

		let filename = format!("{}.{}", Uuid::new_v4(), file_extension);
		let file_path = Path::new(&self.upload_dir).join(&filename);
		let relative_path = format!("{}/{}", self.upload_dir, filename);
		let url = format!("{}/{}", self.base_url, filename);

		let mut file = File::create(&file_path).map_err(|e| {
			tracing::error!("Failed to create file: {}", e);
			AppError::InternalError
		})?;

		file.write_all(&file_data).map_err(|e| {
			tracing::error!("Failed to write file: {}", e);
			AppError::InternalError
		})?;

		let metadata = file.metadata().map_err(|e| {
			tracing::error!("Failed to get file metadata: {}", e);
			AppError::InternalError
		})?;

		let file_metadata = serde_json::json!({
			"created": metadata.created().map(|t| t.elapsed().unwrap().as_secs()).unwrap_or(0),
			"modified": metadata.modified().map(|t| t.elapsed().unwrap().as_secs()).unwrap_or(0),
			"accessed": metadata.accessed().map(|t| t.elapsed().unwrap().as_secs()).unwrap_or(0),
			"is_dir": metadata.is_dir(),
			"is_file": metadata.is_file(),
			"permissions": {
				"readonly": metadata.permissions().readonly(),
			}
		});

		Ok(FileInfo {
			filename,
			original_name: original_name.to_string(),
			path: relative_path,
			url,
			metadata: file_metadata,
		})
	}

	async fn delete_file(&self, file_path: &str) -> Result<(), AppError> {
		let path = Path::new(file_path);
		if path.exists() {
			tokio_fs::remove_file(path).await.map_err(|e| {
				tracing::error!("Failed to delete file: {}", e);
				AppError::InternalError
			})?;
		}

		Ok(())
	}
}

pub fn get_storage_strategy(storage_type: &str, app_config: &dyn IAppConfig) -> Box<dyn IStorageStrategy> {
	match storage_type {
		"local" | _ => Box::new(LocalStorageStrategyImpl::new(
			app_config.get_config().files.upload_dir.clone(),
			app_config.get_config().files.base_url.clone(),
		)),
	}
}
