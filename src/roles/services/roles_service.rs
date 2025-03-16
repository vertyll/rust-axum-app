use crate::common::error::app_error::AppError;
use crate::roles::entities::role::Model as RoleModel;
use crate::roles::repositories::roles_repository::{RolesRepository, RolesRepositoryTrait};
use async_trait::async_trait;
use sea_orm::DatabaseConnection;
use std::sync::Arc;

#[derive(Clone)]
pub struct RolesService {
	roles_repository: Arc<dyn RolesRepositoryTrait>,
}

impl RolesService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			roles_repository: Arc::new(RolesRepository::new(db)),
		}
	}
}

#[async_trait]
pub trait RolesServiceTrait: Send + Sync {
	async fn find_all(&self) -> Result<Vec<RoleModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError>;
	async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError>;
	async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError>;
	async fn update(&self, id: i32, name: Option<String>, description: Option<String>) -> Result<RoleModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
}

#[async_trait]
impl RolesServiceTrait for RolesService {
	async fn find_all(&self) -> Result<Vec<RoleModel>, AppError> {
		self.roles_repository.find_all().await
	}

	async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError> {
		self.roles_repository.find_by_id(id).await
	}

	async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError> {
		self.roles_repository.find_by_name(name).await
	}

	async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError> {
		let existing_role = self.roles_repository.find_by_name(&name).await;
		if let Ok(_role) = existing_role {
			return Err(AppError::ValidationError({
				let mut errors = validator::ValidationErrors::new();
				errors.add("name", validator::ValidationError::new("already_exists"));
				errors
			}));
		}

		self.roles_repository.create(name, description).await
	}

	async fn update(&self, id: i32, name: Option<String>, description: Option<String>) -> Result<RoleModel, AppError> {
		if let Some(ref new_name) = name {
			let existing_role = self.roles_repository.find_by_name(new_name).await;
			if let Ok(role) = existing_role {
				if role.id != id {
					return Err(AppError::ValidationError({
						let mut errors = validator::ValidationErrors::new();
						errors.add("name", validator::ValidationError::new("already_exists"));
						errors
					}));
				}
			}
		}

		self.roles_repository.update(id, name, description).await
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		self.roles_repository.delete(id).await
	}
}
