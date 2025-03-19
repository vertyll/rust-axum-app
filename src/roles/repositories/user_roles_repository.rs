use std::sync::Arc;
use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::roles;
use crate::roles::entities::roles::{self as role_entity, Entity as Role};
use crate::roles::entities::user_roles::{self, Entity as UserRole, Model as UserRoleModel};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set,
};
use crate::di::DatabaseConnectionTrait;

#[derive(Clone)]
pub struct UserRolesRepository {
	db_connection: Arc<dyn DatabaseConnectionTrait>,
}

impl UserRolesRepository {
	pub fn new(db_connection: Arc<dyn DatabaseConnectionTrait>) -> Self {
		Self { db_connection }
	}
}

#[async_trait]
pub trait UserRolesRepositoryTrait: Send + Sync {
	fn get_db(&self) -> &DatabaseConnection;
	async fn find_user_roles(&self, user_id: i32) -> Result<Vec<role_entity::Model>, AppError>;
	async fn assign_user_role_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<UserRoleModel, AppError>;
	async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError>;
	async fn has_role(&self, user_id: i32, role: RoleEnum) -> Result<bool, AppError>;
}

#[async_trait]
impl UserRolesRepositoryTrait for UserRolesRepository {
	fn get_db(&self) -> &DatabaseConnection {
		self.db_connection.get_connection()
	}
	async fn find_user_roles(&self, user_id: i32) -> Result<Vec<role_entity::Model>, AppError> {
		let user_roles = UserRole::find()
			.filter(user_roles::Column::UserId.eq(user_id))
			.find_with_related(Role)
			.all(self.get_db())
			.await?;

		let roles = user_roles.into_iter().flat_map(|(_, roles)| roles).collect();

		Ok(roles)
	}

	async fn assign_user_role_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<UserRoleModel, AppError> {
		use sea_orm::{EntityTrait, QueryFilter};

		let user_role = roles::Entity::find()
			.filter(roles::Column::Name.eq(RoleEnum::User.as_str()))
			.one(transaction)
			.await
			.map_err(|_| AppError::InternalError)?
			.ok_or(AppError::InternalError)?;

		let user_role_model = user_roles::ActiveModel {
			id: Default::default(),
			user_id: Set(user_id),
			role_id: Set(user_role.id),
			created_at: Set(Utc::now().into()),
			updated_at: Set(Some(Utc::now().into())),
		};

		user_role_model
			.insert(transaction)
			.await
			.map_err(|_| AppError::InternalError)
	}

	async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError> {
		let result = UserRole::delete_many()
			.filter(
				Condition::all()
					.add(user_roles::Column::UserId.eq(user_id))
					.add(user_roles::Column::RoleId.eq(role_id)),
			)
			.exec(self.get_db())
			.await?;

		if result.rows_affected == 0 {
			return Err(AppError::NotFound);
		}

		Ok(())
	}

	async fn has_role(&self, user_id: i32, role: RoleEnum) -> Result<bool, AppError> {
		let roles = self.find_user_roles(user_id).await?;
		let has_role = roles.iter().any(|r| r.name == role.as_str());
		Ok(has_role)
	}
}
