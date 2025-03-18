use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::roles::Model as RoleModel;
use crate::roles::entities::user_roles::Model as UserRoleModel;
use crate::roles::repositories::user_roles_repository::IUserRolesRepository;
use async_trait::async_trait;
use sea_orm::DatabaseTransaction;
use shaku::{Component, Interface};
use std::sync::Arc;

#[derive(Component)]
#[shaku(interface = IUserRolesService)]
pub struct UserRolesServiceImpl {
	#[shaku(inject)]
	user_roles_repository: Arc<dyn IUserRolesRepository>,
}

impl UserRolesServiceImpl {
	pub fn new(user_roles_repository: Arc<dyn IUserRolesRepository>) -> Self {
		Self { user_roles_repository }
	}
}

#[async_trait]
pub trait IUserRolesService: Interface {
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
impl IUserRolesService for UserRolesServiceImpl {
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
