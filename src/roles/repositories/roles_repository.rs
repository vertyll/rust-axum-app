use crate::common::error::app_error::AppError;
use crate::roles::entities::role::{self, ActiveModel as RoleActiveModel, Entity as Role, Model as RoleModel};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

#[derive(Clone)]
pub struct RolesRepository {
	db: DatabaseConnection,
}

impl RolesRepository {
	pub fn new(db: DatabaseConnection) -> Self {
		Self { db }
	}

	pub async fn find_all(&self) -> Result<Vec<RoleModel>, AppError> {
		let roles = Role::find().all(&self.db).await?;
		Ok(roles)
	}

	pub async fn find_by_id(&self, id: i32) -> Result<RoleModel, AppError> {
		let role = Role::find_by_id(id).one(&self.db).await?.ok_or(AppError::NotFound)?;
		Ok(role)
	}

	pub async fn find_by_name(&self, name: &str) -> Result<RoleModel, AppError> {
		let role = Role::find()
			.filter(role::Column::Name.eq(name))
			.one(&self.db)
			.await?
			.ok_or(AppError::NotFound)?;
		Ok(role)
	}

	pub async fn create(&self, name: String, description: Option<String>) -> Result<RoleModel, AppError> {
		let now = chrono::Utc::now();

		let role_active_model = RoleActiveModel {
			name: Set(name),
			description: Set(description),
			created_at: Set(now.into()),
			updated_at: Set(now.into()),
			..Default::default()
		};

		let role = role_active_model.insert(&self.db).await?;
		Ok(role)
	}

	pub async fn update(
		&self,
		id: i32,
		name: Option<String>,
		description: Option<String>,
	) -> Result<RoleModel, AppError> {
		let role = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut role_active_model: RoleActiveModel = role.into();

		if let Some(name) = name {
			role_active_model.name = Set(name);
		}

		if let Some(description) = description {
			role_active_model.description = Set(Some(description));
		}

		role_active_model.updated_at = Set(now.into());
		let updated_role = role_active_model.update(&self.db).await?;

		Ok(updated_role)
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		let role = self.find_by_id(id).await?;
		let role_active_model: RoleActiveModel = role.into();
		role_active_model.delete(&self.db).await?;
		Ok(())
	}
}
