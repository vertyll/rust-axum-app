use crate::common::error::app_error::AppError;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::user::{self, ActiveModel as UserActiveModel, Entity as User, Model as UserModel};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, Set};

#[derive(Clone)]
pub struct UsersRepository {
	pub db: DatabaseConnection,
}

impl UsersRepository {
	pub fn new(db: DatabaseConnection) -> Self {
		Self { db }
	}

	pub async fn find_all(&self) -> Result<Vec<UserModel>, AppError> {
		let users = User::find().all(&self.db).await?;

		Ok(users)
	}

	pub async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError> {
		let user = User::find_by_id(id).one(&self.db).await?.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	pub async fn find_by_username(&self, username: &str) -> Result<UserModel, AppError> {
		let user = User::find()
			.filter(user::Column::Username.eq(username))
			.one(&self.db)
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	pub async fn create_in_transaction(
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
			updated_at: Set(now.into()),
			..Default::default()
		};

		let user = user_active_model.insert(transaction).await?;

		Ok(user)
	}

	pub async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError> {
		let user = self.find_by_id(id).await?;
		let now = chrono::Utc::now();

		let mut user_active_model: UserActiveModel = user.into();

		if let Some(username) = dto.username {
			user_active_model.username = Set(username);
		}

		if let Some(email) = dto.email {
			user_active_model.email = Set(email);
		}

		user_active_model.updated_at = Set(now.into());

		let updated_user = user_active_model.update(&self.db).await?;

		Ok(updated_user)
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		let user = self.find_by_id(id).await?;
		let user_active_model: UserActiveModel = user.into();

		user_active_model.delete(&self.db).await?;

		Ok(())
	}

	pub async fn find_by_email(&self, email: &str) -> Result<UserModel, AppError> {
		let user = User::find()
			.filter(user::Column::Email.eq(email))
			.one(&self.db)
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}
}
