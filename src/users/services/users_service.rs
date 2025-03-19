use crate::auth::dto::change_email_dto::ChangeEmailDto;
use crate::auth::dto::change_password_dto::ChangePasswordDto;
use crate::auth::dto::forgot_password_dto::ForgotPasswordDto;
use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::dto::reset_password_dto::ResetPasswordDto;
use crate::auth::services::auth_service::AuthResponse;
use crate::auth::services::confirmation_token_service::{
	ConfirmationTokenService, ConfirmationTokenServiceTrait, TokenType,
};
use crate::common::error::app_error::AppError;
use crate::config::app_config::AppConfig;
use crate::di::AppConfigTrait;
use crate::emails::services::emails_service::{EmailsService, EmailsServiceTrait};
use crate::i18n::setup::translate;
use crate::roles::services::user_roles_service::{UserRolesService, UserRolesServiceTrait};
use crate::users::dto::create_user_dto::CreateUserDto;
use crate::users::dto::update_user_dto::UpdateUserDto;
use crate::users::entities::users::{self, Entity as User, Model as UserModel};
use crate::users::entities::users_email_history::{self, ActiveModel as UserEmailHistoryActiveModel};
use crate::users::repositories::users_repository::{UsersRepository, UsersRepositoryTrait};
use argon2::{
	Argon2,
	password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use async_trait::async_trait;
use chrono::Utc;
use sea_orm::EntityTrait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionTrait};
use std::sync::Arc;

#[derive(Clone)]
pub struct UsersService {
	pub users_repository: Arc<dyn UsersRepositoryTrait>,
	user_roles_service: Arc<dyn UserRolesServiceTrait>,
	email_service: Arc<dyn EmailsServiceTrait>,
	confirmation_token_service: Arc<dyn ConfirmationTokenServiceTrait>,
	app_config: Arc<dyn AppConfigTrait>,
	confirmation_token_expires_in: i64,
}

impl UsersService {
	pub fn new(
		users_repository: Arc<dyn UsersRepositoryTrait>,
		user_roles_service: Arc<dyn UserRolesServiceTrait>,
		email_service: Arc<dyn EmailsServiceTrait>,
		confirmation_token_service: Arc<dyn ConfirmationTokenServiceTrait>,
		app_config: Arc<dyn AppConfigTrait>,
	) -> Self {
		let confirmation_token_expires_in = app_config.get_config().security.tokens.confirmation_token.expires_in;
		Self {
			users_repository,
			user_roles_service,
			email_service,
			confirmation_token_service,
			app_config,
			confirmation_token_expires_in,
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
	async fn confirm_email(&self, token: &str) -> Result<(), AppError>;
	async fn request_reset_password(&self, dto: ForgotPasswordDto) -> Result<(), AppError>;
	async fn reset_password(&self, dto: ResetPasswordDto) -> Result<(), AppError>;
	async fn change_password(&self, user_id: i32, dto: ChangePasswordDto) -> Result<(), AppError>;
	async fn request_email_change(&self, user_id: i32, dto: ChangeEmailDto) -> Result<(), AppError>;
	async fn confirm_email_change(&self, token: &str) -> Result<(), AppError>;
	async fn send_confirmation_email(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		email: &str,
		username: &str,
	) -> Result<(), AppError>;
	async fn create_email_history(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		old_email: &str,
		new_email: &str,
	) -> Result<(), AppError>;
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

			let token = self
				.confirmation_token_service
				.generate_email_confirmation_token(user.id, &user.email)
				.await?;

			self.email_service
				.send_email_confirmation(&user.email, &user.username, &token)
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
		let now = Utc::now();

		let mut user_active_model = users::ActiveModel {
			username: Set(dto.username),
			email: Set(dto.email),
			password_hash: Set(password_hash),
			is_email_confirmed: Set(false),
			created_at: Set(now.into()),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		let user = user_active_model.insert(transaction).await?;

		Ok(user)
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

		if !user.is_active {
			return Err(AppError::AuthenticationError(translate("auth.errors.account_inactive")));
		}

		if !user.is_email_confirmed {
			return Err(AppError::AuthenticationError(translate(
				"auth.errors.email_not_confirmed",
			)));
		}

		if self.verify_password(password, &user.password_hash)? {
			Ok(user)
		} else {
			Err(AppError::AuthenticationError(translate(
				"auth.errors.invalid_credentials",
			)))
		}
	}

	async fn confirm_email(&self, token: &str) -> Result<(), AppError> {
		let claims = self.confirmation_token_service.validate_token(token).await?;
		let user_id = claims.sub;
		let user = self.users_repository.find_by_id(user_id).await?;

		if user.is_email_confirmed {
			let mut errors = validator::ValidationErrors::new();
			errors.add(
				"email",
				validator::ValidationError::new("already_confirmed")
					.with_message(translate("auth.errors.email_already_confirmed").into()),
			);
			return Err(AppError::ValidationError(errors));
		}

		let claims = self
			.confirmation_token_service
			.validate_stored_token(
				token,
				user.email_confirmation_token.as_deref(),
				user.email_confirmation_token_expiry.map(|dt| dt.into()),
				TokenType::EmailConfirmation,
			)
			.await?;

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let mut user_active_model = users::ActiveModel {
			id: Set(claims.sub),
			is_email_confirmed: Set(true),
			email_confirmation_token: Set(None),
			email_confirmation_token_expiry: Set(None),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.update(&transaction).await?;
		transaction.commit().await?;

		Ok(())
	}

	async fn request_reset_password(&self, dto: ForgotPasswordDto) -> Result<(), AppError> {
		let user = self.users_repository.find_by_email(&dto.email).await?;

		let token = self
			.confirmation_token_service
			.generate_password_reset_token(user.id, &user.email)
			.await?;

		let expiry = Utc::now() + chrono::Duration::seconds(self.confirmation_token_expires_in);
		let expiry_sea_orm = expiry.into();

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let mut user_active_model = users::ActiveModel {
			id: Set(user.id),
			password_reset_token: Set(Some(token.clone())),
			password_reset_token_expiry: Set(Some(expiry_sea_orm)),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.update(&transaction).await?;

		match self
			.email_service
			.send_password_reset(&user.email, &user.username, &token)
			.await
		{
			Ok(_) => {
				transaction.commit().await?;
				Ok(())
			}
			Err(e) => {
				transaction.rollback().await?;
				Err(e)
			}
		}
	}

	async fn reset_password(&self, dto: ResetPasswordDto) -> Result<(), AppError> {
		let claims = self.confirmation_token_service.validate_token(&dto.token).await?;
		let user_id = claims.sub;

		let user = self.users_repository.find_by_id(user_id).await?;

		let _claims = self
			.confirmation_token_service
			.validate_stored_token(
				&dto.token,
				user.password_reset_token.as_deref(),
				user.password_reset_token_expiry.map(|dt| dt.into()),
				TokenType::PasswordReset,
			)
			.await?;

		let password_hash = self.hash_password(&dto.password)?;

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let mut user_active_model = users::ActiveModel {
			id: Set(user_id),
			password_hash: Set(password_hash),
			password_reset_token: Set(None),
			password_reset_token_expiry: Set(None),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.update(&transaction).await?;
		transaction.commit().await?;

		Ok(())
	}

	async fn change_password(&self, user_id: i32, dto: ChangePasswordDto) -> Result<(), AppError> {
		let user = self.users_repository.find_by_id(user_id).await?;
		if !self.verify_password(&dto.current_password, &user.password_hash)? {
			let mut errors = validator::ValidationErrors::new();
			errors.add(
				"current_password",
				validator::ValidationError::new("invalid")
					.with_message(translate("users.errors.invalid_current_password").into()),
			);
			return Err(AppError::ValidationError(errors));
		}

		let password_hash = self.hash_password(&dto.new_password)?;

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let mut user_active_model = users::ActiveModel {
			password_hash: Set(password_hash),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.id = Set(user_id);

		user_active_model.update(&transaction).await?;

		transaction.commit().await?;

		Ok(())
	}

	async fn request_email_change(&self, user_id: i32, dto: ChangeEmailDto) -> Result<(), AppError> {
		let user = self.users_repository.find_by_id(user_id).await?;

		if user.email == dto.email {
			let mut errors = validator::ValidationErrors::new();
			errors.add(
				"email",
				validator::ValidationError::new("same_email")
					.with_message(translate("users.errors.new_email_same_as_current").into()),
			);
			return Err(AppError::ValidationError(errors));
		}

		if let Ok(existing_user) = self.users_repository.find_by_email(&dto.email).await {
			if existing_user.id != user_id {
				let mut errors = validator::ValidationErrors::new();
				errors.add(
					"email",
					validator::ValidationError::new("already_exists")
						.with_message(translate("users.errors.email_already_exists").into()),
				);
				return Err(AppError::ValidationError(errors));
			}
		}

		let token = self
			.confirmation_token_service
			.generate_email_change_token(user.id, &user.email, &dto.email)
			.await?;

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let expiry = now + chrono::Duration::seconds(self.confirmation_token_expires_in);

		let mut user_active_model = users::ActiveModel {
			email_change_token: Set(Some(token.clone())),
			email_change_token_expiry: Set(Some(expiry.into())),
			pending_email: Set(Some(dto.email.clone())),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.id = Set(user_id);
		user_active_model.update(&transaction).await?;

		self.email_service
			.send_email_change_confirmation(&user.email, &user.username, &token)
			.await?;

		transaction.commit().await?;
		Ok(())
	}

	async fn confirm_email_change(&self, token: &str) -> Result<(), AppError> {
		let claims = self.confirmation_token_service.validate_token(token).await?;
		let user_id = claims.sub;

		let user = self.users_repository.find_by_id(user_id).await?;

		let claims = self
			.confirmation_token_service
			.validate_stored_token(
				token,
				user.email_change_token.as_deref(),
				user.email_change_token_expiry.map(|dt| dt.into()),
				TokenType::EmailChange,
			)
			.await?;

		let new_email = claims
			.new_email
			.as_ref()
			.ok_or_else(|| AppError::AuthorizationError(translate("auth.errors.invalid_token").into()))?;

		let db = self.users_repository.get_db();
		let transaction = db.begin().await?;

		let now = Utc::now();
		let mut user_active_model = users::ActiveModel {
			id: Set(user_id),
			email: Set(new_email.clone()),
			email_change_token: Set(None),
			email_change_token_expiry: Set(None),
			pending_email: Set(None),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		user_active_model.update(&transaction).await?;

		self.create_email_history(&transaction, user_id, &user.email, new_email)
			.await?;

		transaction.commit().await?;

		Ok(())
	}

	async fn send_confirmation_email(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		email: &str,
		username: &str,
	) -> Result<(), AppError> {
		let token = self
			.confirmation_token_service
			.generate_email_confirmation_token(user_id, email)
			.await?;

		let expiry = Utc::now() + chrono::Duration::seconds(self.confirmation_token_expires_in);
		let expiry_sea_orm = expiry.into();

		let user = users::Entity::find_by_id(user_id)
			.one(transaction)
			.await?
			.ok_or(AppError::NotFound)?;

		let mut user_active_model: users::ActiveModel = user.into();
		user_active_model.email_confirmation_token = Set(Some(token.clone()));
		user_active_model.email_confirmation_token_expiry = Set(Some(expiry_sea_orm));
		user_active_model.updated_at = Set(Some(Utc::now().into()));

		user_active_model.update(transaction).await?;

		self.email_service
			.send_email_confirmation(email, username, &token)
			.await?;

		Ok(())
	}

	async fn create_email_history(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		old_email: &str,
		new_email: &str,
	) -> Result<(), AppError> {
		let now = Utc::now();

		let email_history = UserEmailHistoryActiveModel {
			user_id: Set(user_id),
			old_email: Set(old_email.to_string()),
			new_email: Set(new_email.to_string()),
			email_change_at: Set(now.into()),
			created_at: Set(now.into()),
			updated_at: Set(Some(now.into())),
			..Default::default()
		};

		email_history.insert(transaction).await?;
		Ok(())
	}
}
