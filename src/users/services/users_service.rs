use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::services::auth_service::AuthResponse;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::i18n::setup::translate;
use crate::roles::services::user_roles_service::{UserRolesService, UserRolesServiceTrait};
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::users::{self, Entity as User, Model as UserModel};
use crate::users::repositories::users_repository::{UsersRepository, UsersRepositoryTrait};
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct UsersService {
	pub users_repository: Arc<dyn UsersRepositoryTrait>,
	user_roles_service: Arc<dyn UserRolesServiceTrait>,
}

impl UsersService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			users_repository: Arc::new(UsersRepository::new(db.clone())),
			user_roles_service: Arc::new(UserRolesService::new(db)),
		}
	}

	fn hash_password(&self, password: &str) -> Result<String, AppError> {
		let salt = SaltString::generate(&mut OsRng);
		let argon2 = Argon2::default();

		let password_hash = argon2
			.hash_password(password.as_bytes(), &salt)
			.map_err(|e| {
				tracing::error!("Error hashing password: {}", e);
				AppError::InternalError
			})?
			.to_string();

		Ok(password_hash)
	}

	fn verify_password(&self, password: &str, hash: &str) -> Result<bool, AppError> {
		let parsed_hash = PasswordHash::new(hash).map_err(|e| {
			tracing::error!("Error parsing password hash: {}", e);
			AppError::InternalError
		})?;

		Ok(Argon2::default()
			.verify_password(password.as_bytes(), &parsed_hash)
			.is_ok())
	}
}

#[async_trait]
pub trait UsersServiceTrait: Send + Sync {
	async fn begin_transaction(&self) -> Result<DatabaseTransaction, AppError>;
	async fn find_all(&self) -> Result<Vec<UserModel>, AppError>;
	async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError>;
	async fn create(&self, dto: CreateUserDto) -> Result<UserModel, AppError>;
	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateUserDto,
	) -> Result<UserModel, AppError>;
	async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError>;
	async fn delete(&self, id: i32) -> Result<(), AppError>;
	async fn login(&self, username: &str, password: &str) -> Result<UserModel, AppError>;
}

#[async_trait]
impl UsersServiceTrait for UsersService {
	async fn begin_transaction(&self) -> Result<DatabaseTransaction, AppError> {
		Ok(self.users_repository.get_db().begin().await?)
	}

	async fn find_all(&self) -> Result<Vec<UserModel>, AppError> {
		self.users_repository.find_all().await
	}

	async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError> {
		self.users_repository.find_by_id(id).await
	}

	async fn create(&self, dto: CreateUserDto) -> Result<UserModel, AppError> {
		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let result = self.create_in_transaction(&transaction, dto).await;

		if let Ok(user) = &result {
			self.user_roles_service
				.assign_user_role_in_transaction(&transaction, user.id)
				.await?;
		}

		match result {
			Ok(response) => {
				transaction.commit().await?;
				Ok(response)
			}
			Err(e) => {
				transaction.rollback().await?;
				Err(e)
			}
		}
	}

	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateUserDto,
	) -> Result<UserModel, AppError> {
		let mut validation_errors = validator::ValidationErrors::new();
		let mut has_errors = false;

		let user_email = dto.email.clone();
		if let Ok(_) = self.users_repository.find_by_email(&user_email).await {
			has_errors = true;
			validation_errors.add(
				"email",
				validator::ValidationError::new("already_exists")
					.with_message(translate("users.errors.user_already_exists").into()),
			);
		}

		let username = dto.username.clone();
		if let Ok(_) = self.users_repository.find_by_username(&username).await {
			has_errors = true;
			validation_errors.add(
				"username",
				validator::ValidationError::new("already_exists")
					.with_message(translate("users.errors.username_already_exists").into()),
			);
		}

		if has_errors {
			return Err(AppError::ValidationError(validation_errors));
		}

		let password_hash = self.hash_password(&dto.password)?;
		self.users_repository
			.create_in_transaction(transaction, dto, password_hash)
			.await
	}

	async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError> {
		let _existing_user = self.users_repository.find_by_id(id).await?;

		self.users_repository.update(id, dto).await
	}

	async fn delete(&self, id: i32) -> Result<(), AppError> {
		let _existing_user = self.users_repository.find_by_id(id).await?;

		self.users_repository.delete(id).await
	}

	async fn login(&self, username: &str, password: &str) -> Result<UserModel, AppError> {
		let user = self.users_repository.find_by_username(username).await?;

		if self.verify_password(password, &user.password_hash)? {
			Ok(user)
		} else {
			Err(AppError::AuthenticationError("Invalid credentials".to_string()))
		}
	}
}
