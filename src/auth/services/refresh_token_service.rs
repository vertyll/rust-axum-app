use crate::auth::dto::access_token_dto::AccessTokenDto;
use crate::auth::repositories::refresh_token_repository::IRefreshTokenRepository;
use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::di::IAppConfig;
use crate::i18n::setup::translate;
use crate::roles::services::user_roles_service::IUserRolesService;
use async_trait::async_trait;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::DatabaseTransaction;
use serde::{Deserialize, Serialize};
use shaku::{Component, Interface};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: i32,
	pub username: String,
	pub email: String,
	pub roles: Vec<RoleEnum>,
	pub exp: i64,
	pub iat: i64,
}

#[derive(Component)]
#[shaku(interface = IRefreshTokenService)]
pub struct RefreshTokenServiceImpl {
	#[shaku(inject)]
	refresh_token_repository: Arc<dyn IRefreshTokenRepository>,
	#[shaku(inject)]
	user_roles_service: Arc<dyn IUserRolesService>,
	#[shaku(inject)]
	app_config: Arc<dyn IAppConfig>,
}

impl RefreshTokenServiceImpl {
	pub fn new(
		refresh_token_repository: Arc<dyn IRefreshTokenRepository>,
		user_roles_service: Arc<dyn IUserRolesService>,
		app_config: Arc<dyn IAppConfig>,
	) -> Self {
		Self {
			refresh_token_repository,
			user_roles_service,
			app_config,
		}
	}
	async fn generate_access_token(&self, user_id: i32) -> Result<String, AppError> {
		let user = self.refresh_token_repository.find_user_by_id(user_id).await?;

		let user_roles = self.user_roles_service.get_user_roles(user_id).await?;

		let role_enums: Vec<RoleEnum> = user_roles
			.into_iter()
			.filter_map(|role| RoleEnum::from_str(&role.name))
			.collect();

		let now = Utc::now();
		let expires_at =
			now + chrono::Duration::seconds(self.app_config.get_config().security.tokens.jwt_access_token.expires_in);

		let claims = Claims {
			sub: user_id,
			username: user.username.clone(),
			email: user.email.clone(),
			roles: role_enums,
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
		};

		encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(
				self.app_config
					.get_config()
					.security
					.tokens
					.jwt_access_token
					.secret
					.as_bytes(),
			),
		)
		.map_err(|_| AppError::InternalError)
	}
}

#[async_trait]
pub trait IRefreshTokenService: Interface {
	async fn generate_refresh_token(&self, user_id: i32) -> Result<String, AppError>;
	async fn generate_refresh_token_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<String, AppError>;
	async fn refresh_token(&self, user_id: i32, refresh_token: String) -> Result<AccessTokenDto, AppError>;
	async fn invalidate_refresh_token(&self, user_id: i32, refresh_token: String) -> Result<(), AppError>;
	async fn invalidate_all_user_tokens(&self, user_id: i32) -> Result<(), AppError>;
	async fn clean_expired_tokens(&self) -> Result<(), AppError>;
}

#[async_trait]
impl IRefreshTokenService for RefreshTokenServiceImpl {
	async fn generate_refresh_token(&self, user_id: i32) -> Result<String, AppError> {
		let (_, token) = self
			.refresh_token_repository
			.create(
				user_id,
				self.app_config
					.get_config()
					.security
					.tokens
					.jwt_refresh_token
					.expires_in,
			)
			.await?;
		Ok(token)
	}

	async fn generate_refresh_token_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<String, AppError> {
		let (_, token) = self
			.refresh_token_repository
			.create_in_transaction(
				transaction,
				user_id,
				self.app_config
					.get_config()
					.security
					.tokens
					.jwt_refresh_token
					.expires_in,
			)
			.await?;
		Ok(token)
	}

	async fn refresh_token(&self, user_id: i32, refresh_token: String) -> Result<AccessTokenDto, AppError> {
		let token = self
			.refresh_token_repository
			.find_by_token_and_user_id(&refresh_token, user_id)
			.await
			.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_refresh_token")))?;

		if !self.refresh_token_repository.is_token_valid(&token).await {
			return Err(AppError::AuthenticationError(translate(
				"auth.errors.expired_refresh_token",
			)));
		}

		let access_token = self.generate_access_token(user_id).await?;

		Ok(AccessTokenDto { access_token })
	}

	async fn invalidate_refresh_token(&self, user_id: i32, refresh_token: String) -> Result<(), AppError> {
		self.refresh_token_repository
			.delete_by_token_and_user_id(&refresh_token, user_id)
			.await
	}

	async fn invalidate_all_user_tokens(&self, user_id: i32) -> Result<(), AppError> {
		self.refresh_token_repository.delete_all_by_user_id(user_id).await
	}

	async fn clean_expired_tokens(&self) -> Result<(), AppError> {
		self.refresh_token_repository.delete_expired().await
	}
}
