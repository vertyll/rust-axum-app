use crate::common::error::error::AppError;
use crate::i18n::setup::translate;
use crate::users::dto::create_user::CreateUserDto;
use crate::users::dto::update_user::UpdateUserDto;
use crate::users::entities::user::User;
use crate::users::repositories::users_repository::UsersRepository;
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct UsersService {
	repository: UsersRepository,
}

impl UsersService {
	pub fn new(db_pool: PgPool) -> Self {
		Self {
			repository: UsersRepository::new(db_pool),
		}
	}

	pub async fn find_all(&self) -> Result<Vec<User>, AppError> {
		self.repository.find_all().await
	}

	pub async fn find_by_id(&self, id: i32) -> Result<User, AppError> {
		self.repository.find_by_id(id).await
	}

	pub async fn create(&self, dto: CreateUserDto) -> Result<User, AppError> {
		let user_email = dto.email.clone();

		let existing_user = self.repository.find_by_email(&user_email).await?;
		if existing_user.email == user_email {
			return Err(AppError::ValidationError({
				let mut errors = validator::ValidationErrors::new();
				errors.add(
					"email",
					validator::ValidationError::new("already_exists")
						.with_message(translate("users.errors.user_already_exists").into()),
				);
				errors
			}));
		}

		let password_hash = hash_password(&dto.password)?;

		self.repository.create(dto, password_hash).await
	}

	pub async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<User, AppError> {
		let _existing_user = self.repository.find_by_id(id).await?;

		self.repository.update(id, dto).await
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		let _existing_user = self.repository.find_by_id(id).await?;

		self.repository.delete(id).await
	}

	pub async fn login(&self, username: &str, password: &str) -> Result<User, AppError> {
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
