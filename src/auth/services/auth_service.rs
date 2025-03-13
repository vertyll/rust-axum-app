use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header, encode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::auth::dto::login::LoginDto;
use crate::auth::dto::register::RegisterDto;
use crate::common::error::error::AppError;
use crate::users::entities::user::User;
use crate::users::services::users_service::UsersService;

#[derive(Clone)]
pub struct AuthService {
	users_service: UsersService,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
	pub sub: i32,
	pub username: String,
	pub email: String,
	pub exp: i64,
	pub iat: i64,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
	pub user: User,
	pub token: String,
}

impl AuthService {
	pub fn new(db_pool: PgPool) -> Self {
		Self {
			users_service: UsersService::new(db_pool),
		}
	}

	pub async fn register(&self, dto: RegisterDto) -> Result<AuthResponse, AppError> {
		let create_user_dto = crate::users::dto::create_user::CreateUserDto {
			username: dto.username,
			email: dto.email,
			password: dto.password,
		};

		let user = self.users_service.create(create_user_dto).await?;
		let token = self.generate_token(&user)?;

		Ok(AuthResponse { user, token })
	}

	pub async fn login(&self, dto: LoginDto) -> Result<AuthResponse, AppError> {
		let user = self.users_service.login(&dto.username, &dto.password).await?;
		let token = self.generate_token(&user)?;

		Ok(AuthResponse { user, token })
	}

	fn generate_token(&self, user: &User) -> Result<String, AppError> {
		let now = Utc::now();
		let app_config =
			crate::config::app_config::AppConfig::init().expect("Could not initialize the application configuration");
		let expires_at = now + Duration::seconds(app_config.security.jwt_access_token_expires_in);

		let claims = Claims {
			sub: user.id,
			username: user.username.clone(),
			email: user.email.clone(),
			exp: expires_at.timestamp(),
			iat: now.timestamp(),
		};

		let token = encode(
			&Header::default(),
			&claims,
			&EncodingKey::from_secret(app_config.security.jwt_access_token_secret.as_bytes()),
		)
		.map_err(|_| AppError::InternalError)?;

		Ok(token)
	}
}
