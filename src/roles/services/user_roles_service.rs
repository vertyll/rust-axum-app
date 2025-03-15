use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::role::Model as RoleModel;
use crate::roles::entities::user_role::Model as UserRoleModel;
use crate::roles::repositories::user_roles_repository::UserRolesRepository;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct UserRolesService {
	repository: UserRolesRepository,
}

impl UserRolesService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			repository: UserRolesRepository::new(db),
		}
	}

	pub async fn get_user_roles(&self, user_id: i32) -> Result<Vec<RoleModel>, AppError> {
		self.repository.find_user_roles(user_id).await
	}

	pub async fn assign_user_role(&self, user_id: i32) -> Result<UserRoleModel, AppError> {
		self.repository.assign_user_role(user_id).await
	}

	pub async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError> {
		self.repository.remove_role(user_id, role_id).await
	}

	pub async fn has_role(&self, user_id: i32, role_name: RoleEnum) -> Result<bool, AppError> {
		self.repository.has_role(user_id, role_name).await
	}
}
