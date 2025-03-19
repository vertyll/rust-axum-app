use crate::common::error::app_error::AppError;
use crate::di::module::IDatabaseConnection;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::users::{self, ActiveModel as UserActiveModel, Entity as User, Model as UserModel};
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use shaku::{Component, Interface};
use std::sync::Arc;

#[derive(Component)]
#[shaku(interface = IUsersRepository)]
pub struct UsersRepositoryImpl {
	#[shaku(inject)]
	pub db_provider: Arc<dyn IDatabaseConnection>,
}

#[async_trait]
pub trait IUsersRepository: Interface {
	fn get_db(&self) -> &DatabaseConnection;
	async fn find_all(&self) -> Result<Vec<UserModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError>;
	async fn find_by_username(&self, username: &str) -> Result<UserModel, AppError>;
	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateUserDto,
		password_hash: String,
	) -> Result<UserModel, AppError>;
	async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
	async fn find_by_email(&self, email: &str) -> Result<UserModel, AppError>;
}

#[async_trait]
impl IUsersRepository for UsersRepositoryImpl {
	fn get_db(&self) -> &DatabaseConnection {
		self.db_provider.get_connection()
	}

	async fn find_all(&self) -> Result<Vec<UserModel>, AppError> {
		let users = User::find().all(self.get_db()).await?;

		Ok(users)
	}

	async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError> {
		let user = User::find_by_id(id)
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	async fn find_by_username(&self, username: &str) -> Result<UserModel, AppError> {
		let user = User::find()
			.filter(users::Column::Username.eq(username))
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateUserDto,
		password_hash: String,
	) -> Result<UserModel, AppError> {
		let now = chrono::Utc::now();

		let user_active_model = UserActiveModel {
			username: Set(dto.username),
			email: Set(dto.email),
			password_hash: Set(password_hash),
			created_at: Set(now.into()),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		let user = user_active_model.insert(transaction).await?;

		Ok(user)
	}

	async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError> {
		let user = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut user_active_model: UserActiveModel = user.into();

		if let Some(username) = dto.username {
			user_active_model.username = Set(username);
		}

		if let Some(email) = dto.email {
			user_active_model.email = Set(email);
		}

		user_active_model.updated_at = Set(Some(now.into()));

		let updated_user = user_active_model.update(self.get_db()).await?;

		Ok(updated_user)
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		let user = self.find_by_id(id).await?;
		let mut user_active_model: UserActiveModel = user.into();

		user_active_model.is_active = Set(false);
		let now = chrono::Utc::now();
		user_active_model.updated_at = Set(Some(now.into()));

		user_active_model.update(self.get_db()).await?;

		Ok(())
	}

	async fn find_by_email(&self, email: &str) -> Result<UserModel, AppError> {
		let user = User::find()
			.filter(users::Column::Email.eq(email))
			.one(self.get_db())
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}
}
