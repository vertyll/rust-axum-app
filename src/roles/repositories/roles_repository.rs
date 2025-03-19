use crate::common::error::app_error::AppError;
use crate::di::DatabaseConnectionTrait;
use crate::roles::entities::roles::{self, ActiveModel as RoleActiveModel, Entity as Role, Model as RoleModel};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use std::sync::Arc;

#[derive(Clone)]
pub struct RolesRepository {
	db_connection: Arc<dyn DatabaseConnectionTrait>,
}

impl RolesRepository {
	pub fn new(db_connection: Arc<dyn DatabaseConnectionTrait>) -> Self {
		Self { db_connection }
	}
}

#[async_trait]
pub trait RolesRepositoryTrait: Send + Sync {
	fn get_db(&self) -> &sea_orm::DatabaseConnection;
	async fn find_all(&self) -> Result<Vec<RoleModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError>;
	async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError>;
	async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError>;
	async fn update(&self, id: i32, name: Option<String>, description: Option<String>) -> Result<RoleModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
}

#[async_trait]
impl RolesRepositoryTrait for RolesRepository {
	fn get_db(&self) -> &DatabaseConnection {
		self.db_connection.get_connection()
	}

	async fn find_all(&self) -> Result<Vec<RoleModel>, AppError> {
		let roles = Role::find().all(self.get_db()).await?;
		Ok(roles)
	}

	async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError> {
		let role = Role::find_by_id(id)
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;
		Ok(role)
	}

	async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError> {
		let role = Role::find()
			.filter(roles::Column::Name.eq(name))
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;
		Ok(role)
	}

	async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError> {
		let now = chrono::Utc::now();

		let role_active_model = RoleActiveModel {
			name: Set(name),
			description: Set(description),
			created_at: Set(now.into()),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		let role = role_active_model.insert(self.get_db()).await?;
		Ok(role)
	}

	async fn update(&self, id: i32, name: Option<String>, description: Option<String>) -> Result<RoleModel, AppError> {
		let role = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut role_active_model: RoleActiveModel = role.into();

		if let Some(name) = name {
			role_active_model.name = Set(name);
		}

		if let Some(description) = description {
			role_active_model.description = Set(Some(description));
		}

		role_active_model.updated_at = Set(Some(now.into()));
		let updated_role = role_active_model.update(self.get_db()).await?;

		Ok(updated_role)
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		let role = self.find_by_id(id).await?;
		let role_active_model: RoleActiveModel = role.into();
		role_active_model.delete(self.get_db()).await?;
		Ok(())
	}
}
