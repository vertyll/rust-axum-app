use crate::common::error::app_error::AppError;
use crate::roles::entities::role::Model as RoleModel;
use crate::roles::repositories::roles_repository::RolesRepository;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct RolesService {
	repository: RolesRepository,
}

impl RolesService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			repository: RolesRepository::new(db),
		}
	}

	pub async fn find_all(&self) -> Result<Vec<RoleModel>, AppError> {
		self.repository.find_all().await
	}

	pub async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError> {
		self.repository.find_by_id(id).await
	}

	pub async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError> {
		self.repository.find_by_name(name).await
	}

	pub async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError> {
		let existing_role = self.repository.find_by_name(&name).await;
		if let Ok(_role) = existing_role {
			return Err(AppError::ValidationError({
				let mut errors = validator::ValidationErrors::new();
				errors.add("name", validator::ValidationError::new("already_exists"));
				errors
			}));
		}

		self.repository.create(name, description).await
	}

	pub async fn update(
		&self,
		id: i32,
		name: Option<String>,
		description: Option<String>,
	) -> Result<RoleModel, AppError> {
		if let Some(ref new_name) = name {
			let existing_role = self.repository.find_by_name(new_name).await;
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

		self.repository.update(id, name, description).await
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		self.repository.delete(id).await
	}
}
