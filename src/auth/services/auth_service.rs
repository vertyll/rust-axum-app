use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::{DatabaseTransaction, TransactionTrait};
use serde::{Deserialize, Serialize};

use crate::auth::dto::login_dto::LoginDto;
use crate::auth::dto::register_dto::RegisterDto;
use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::roles::services::user_roles_service::UserRolesService;
use crate::users::entities::user::Model as User;
use crate::users::services::users_service::UsersService;

#[derive(Clone)]
pub struct AuthService {
	users_service: UsersService,
	user_roles_service: UserRolesService,
	jwt_access_token_secret: String,
	jwt_access_token_expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: i32,
	pub username: String,
	pub email: String,
	pub roles: Vec<RoleEnum>,
	pub exp: i64,
	pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
	pub user: User,
	pub token: String,
}

impl AuthService {
	pub fn new(app_state: AppState, jwt_access_token_expires_in: i64) -> Self {
		Self {
			users_service: UsersService::new(app_state.db.clone()),
			user_roles_service: UserRolesService::new(app_state.db.clone()),
			jwt_access_token_secret: app_state.jwt_access_token_secret,
			jwt_access_token_expires_in,
		}
	}

	pub async fn register(&self, dto: RegisterDto) -> Result<AuthResponse, AppError> {
		let db = &self.users_service.repository.db;
		let transaction = db.begin().await?;

		let result = self.register_in_transaction(&transaction, dto).await;

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

	async fn register_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		dto: RegisterDto,
	) -> Result<AuthResponse, AppError> {
		let create_user_dto = crate::users::dto::create_user_dto::CreateUserDto {
			username: dto.username,
			email: dto.email,
			password: dto.password,
		};

		let user = self
			.users_service
			.create_in_transaction(transaction, create_user_dto)
			.await?;

		self.user_roles_service
			.assign_user_role_in_transaction(transaction, user.id)
			.await?;

		let token = self.generate_token(&user).await?;

		Ok(AuthResponse { user, token })
	}

	pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
		let user = self.users_service.login(&dto.username, &dto.password).await?;
		let token = self.generate_token(&user).await?;

		Ok(AuthResponse { user, token })
	}

	async fn generate_token(&self, user: &User) -> Result<String, AppError> {
		let now = Utc::now();
		let expires_at = now + Duration::seconds(self.jwt_access_token_expires_in);

		let user_roles = self.user_roles_service.get_user_roles(user.id).await?;
		let role_enums: Vec<RoleEnum> = user_roles
			.into_iter()
			.filter_map(|role| RoleEnum::from_str(&role.name))
			.collect();

		let claims = Claims {
			sub: user.id,
			username: user.username.clone(),
			email: user.email.clone(),
			roles: role_enums,
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(self.jwt_access_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)?;

		Ok(token)
	}
}
