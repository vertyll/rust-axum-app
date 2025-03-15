use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::roles::entities::role;
use crate::roles::entities::role::{self as role_entity, Entity as Role};
use crate::roles::entities::user_role::{self, Entity as UserRole, Model as UserRoleModel};
use chrono::Utc;
use sea_orm::{
	ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set,
};

#[derive(Clone)]
pub struct UserRolesRepository {
	db: DatabaseConnection,
}

impl UserRolesRepository {
	pub fn new(db: DatabaseConnection) -> Self {
		Self { db }
	}

	pub async fn find_user_roles(&self, user_id: i32) -> Result<Vec<role_entity::Model>, AppError> {
		let user_roles = UserRole::find()
			.filter(user_role::Column::UserId.eq(user_id))
			.find_with_related(Role)
			.all(&self.db)
			.await?;

		let roles = user_roles.into_iter().flat_map(|(_, roles)| roles).collect();

		Ok(roles)
	}

	pub async fn assign_user_role_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<UserRoleModel, AppError> {
		use sea_orm::{EntityTrait, QueryFilter};

		let user_role = role::Entity::find()
			.filter(role::Column::Name.eq(RoleEnum::User.as_str()))
			.one(transaction)
			.await
			.map_err(|_| AppError::InternalError)?
			.ok_or(AppError::InternalError)?;

		let user_role_model = user_role::ActiveModel {
			id: Default::default(),
			user_id: Set(user_id),
			role_id: Set(user_role.id),
			created_at: Set(Utc::now().into()),
			updated_at: Set(Utc::now().into()),
		};

		user_role_model
			.insert(transaction)
			.await
			.map_err(|_| AppError::InternalError)
	}

	pub async fn remove_role(&self, user_id: i32, role_id: i32) -> Result<(), AppError> {
		let result = UserRole::delete_many()
			.filter(
				Condition::all()
					.add(user_role::Column::UserId.eq(user_id))
					.add(user_role::Column::RoleId.eq(role_id)),
			)
			.exec(&self.db)
			.await?;

		if result.rows_affected == 0 {
			return Err(AppError::NotFound);
		}

		Ok(())
	}

	pub async fn has_role(&self, user_id: i32, role: RoleEnum) -> Result<bool, AppError> {
		let roles = self.find_user_roles(user_id).await?;
		let has_role = roles.iter().any(|r| r.name == role.as_str());
		Ok(has_role)
	}
}
