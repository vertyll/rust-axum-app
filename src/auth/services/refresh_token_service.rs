use crate::auth::dto::access_token_dto::AccessTokenDto;
use crate::auth::repositories::refresh_token_repository::RefreshTokenRepository;
use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::common::r#struct::token_state::TokenState;
use crate::i18n::setup::translate;
use crate::roles::services::user_roles_service::UserRolesService;
use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::DatabaseTransaction;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: i32,
	pub username: String,
	pub email: String,
	pub roles: Vec<RoleEnum>,
	pub exp: i64,
	pub iat: i64,
}

#[derive(Clone)]
pub struct RefreshTokenService {
	repository: RefreshTokenRepository,
	jwt_access_token_secret: String,
	jwt_access_token_expires_in: i64,
	jwt_refresh_token_expires_in: i64,
}

impl RefreshTokenService {
	pub fn new(app_state: AppState, token_state: TokenState) -> Self {
		Self {
			repository: RefreshTokenRepository::new(app_state.db.clone()),
			jwt_access_token_secret: token_state.jwt_access_token_secret,
			jwt_access_token_expires_in: token_state.jwt_access_token_expires_in,
			jwt_refresh_token_expires_in: token_state.jwt_refresh_token_expires_in,
		}
	}

	pub async fn generate_refresh_token(&self, user_id: i32) -> Result<String, AppError> {
		let (_, token) = self
			.repository
			.create(user_id, self.jwt_refresh_token_expires_in)
			.await?;
		Ok(token)
	}

	pub async fn generate_refresh_token_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
	) -> Result<String, AppError> {
		let (_, token) = self
			.repository
			.create_in_transaction(transaction, user_id, self.jwt_refresh_token_expires_in)
			.await?;
		Ok(token)
	}

	pub async fn refresh_token(&self, user_id: i32, refresh_token: String) -> Result<AccessTokenDto, AppError> {
		let token = self
			.repository
			.find_by_token_and_user_id(&refresh_token, user_id)
			.await
			.map_err(|_| AppError::AuthenticationError(translate("auth.errors.invalid_refresh_token")))?;

		if !self.repository.is_token_valid(&token).await {
			return Err(AppError::AuthenticationError(translate(
				"auth.errors.expired_refresh_token",
			)));
		}

		let access_token = self.generate_access_token(user_id).await?;

		Ok(AccessTokenDto { access_token })
	}

	pub async fn invalidate_refresh_token(&self, user_id: i32, refresh_token: String) -> Result<(), AppError> {
		self.repository
			.delete_by_token_and_user_id(&refresh_token, user_id)
			.await
	}

	async fn generate_access_token(&self, user_id: i32) -> Result<String, AppError> {
		let user = self.repository.find_user_by_id(user_id).await?;

		let user_roles_service = UserRolesService::new(self.repository.db.clone());
		let user_roles = user_roles_service.get_user_roles(user_id).await?;

		let role_enums: Vec<RoleEnum> = user_roles
			.into_iter()
			.filter_map(|role| RoleEnum::from_str(&role.name))
			.collect();

		let now = Utc::now();
		let expires_at = now + chrono::Duration::seconds(self.jwt_access_token_expires_in);

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
			&EncodingKey::from_secret(self.jwt_access_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)
	}

	pub async fn invalidate_all_user_tokens(&self, user_id: i32) -> Result<(), AppError> {
		self.repository.delete_all_by_user_id(user_id).await
	}

	pub async fn clean_expired_tokens(&self) -> Result<(), AppError> {
		self.repository.delete_expired().await
	}
}
