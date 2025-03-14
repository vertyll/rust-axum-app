use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::services::auth_service::AuthResponse;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::i18n::setup::translate;
use crate::roles::services::user_roles_service::UserRolesService;
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::user::{self, Entity as User, Model as UserModel};
use crate::users::repositories::users_repository::UsersRepository;
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

#[derive(Clone)]
pub struct UsersService {
	pub repository: UsersRepository,
	user_roles_service: UserRolesService,
}

impl UsersService {
	pub fn new(db: DatabaseConnection) -> Self {
		Self {
			repository: UsersRepository::new(db.clone()),
			user_roles_service: UserRolesService::new(db.clone()),
		}
	}

	pub async fn find_all(&self) -> Result<Vec<UserModel>, AppError> {
		self.repository.find_all().await
	}

	pub async fn find_by_id(&self, id: i32) -> Result<UserModel, AppError> {
		self.repository.find_by_id(id).await
	}

	pub async fn create(&self, dto: CreateUserDto) -> Result<UserModel, AppError> {
		let db = &self.repository.db;
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

	pub async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: CreateUserDto,
	) -> Result<UserModel, AppError> {
		let mut validation_errors = validator::ValidationErrors::new();
		let mut has_errors = false;

		let user_email = dto.email.clone();
		if let Ok(_) = self.repository.find_by_email(&user_email).await {
			has_errors = true;
			validation_errors.add(
				"email",
				validator::ValidationError::new("already_exists")
					.with_message(translate("users.errors.user_already_exists").into()),
			);
		}

		let username = dto.username.clone();
		if let Ok(_) = self.repository.find_by_username(&username).await {
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

		let password_hash = hash_password(&dto.password)?;
		self.repository
			.create_in_transaction(transaction, dto, password_hash)
			.await
	}

	pub async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<UserModel, AppError> {
		let _existing_user = self.repository.find_by_id(id).await?;

		self.repository.update(id, dto).await
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		let _existing_user = self.repository.find_by_id(id).await?;

		self.repository.delete(id).await
	}

	pub async fn login(&self, username: &str, password: &str) -> Result<UserModel, AppError> {
		let user = self.repository.find_by_username(username).await?;

		if verify_password(password, &user.password_hash)? {
			Ok(user)
		} else {
			Err(AppError::AuthenticationError("Invalid credentials".to_string()))
		}
	}
}

fn hash_password(password: &str) -> Result<String, AppError> {
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

fn verify_password(password: &str, hash: &str) -> Result<bool, AppError> {
	let parsed_hash = PasswordHash::new(hash).map_err(|e| {
		tracing::error!("Error parsing password hash: {}", e);
		AppError::InternalError
	})?;

	Ok(Argon2::default()
		.verify_password(password.as_bytes(), &parsed_hash)
		.is_ok())
}
