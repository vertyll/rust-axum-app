use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::roles::Model as RoleModel;
use crate::roles::entities::user_roles::Model as UserRoleModel;
use crate::roles::repositories::user_roles_repository::{UserRolesRepository, UserRolesRepositoryTrait};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction};
use std::sync::Arc;

#[derive(Clone)]
pub struct UserRolesService {
	user_roles_repository: Arc<dyn UserRolesRepositoryTrait>,
}

impl UserRolesService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			user_roles_repository: Arc::new(UserRolesRepository::new(db)),
		}
	}
}

#[async_trait]
pub trait UserRolesServiceTrait: Send + Sync {
	async fn get_user_roles(&self, user_id: i32) -> Result<Vec<RoleModel>, AppError>;
	async fn assign_user_role_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<UserRoleModel, AppError>;
	async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError>;
	async fn has_role(&self, user_id: i32, role_name: RoleEnum) -> Result<bool, AppError>;
}

#[async_trait]
impl UserRolesServiceTrait for UserRolesService {
	async fn get_user_roles(&self, user_id: i32) -> Result<Vec<RoleModel>, AppError> {
		self.user_roles_repository.find_user_roles(user_id).await
	}

	async fn assign_user_role_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<UserRoleModel, AppError> {
		self.user_roles_repository
			.assign_user_role_in_transaction(transaction, user_id)
			.await
	}

	async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError> {
		self.user_roles_repository.remove_role(user_id, role_id).await
	}

	async fn has_role(&self, user_id: i32, role_name: RoleEnum) -> Result<bool, AppError> {
		self.user_roles_repository.has_role(user_id, role_name).await
	}
}
