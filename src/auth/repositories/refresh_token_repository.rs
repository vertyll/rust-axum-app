use crate::auth::entities::refresh_tokens::{
	self, ActiveModel as RefreshTokenActiveModel, Entity as RefreshToken, Model as RefreshTokenModel,
};
use crate::common::error::app_error::AppError;
use crate::di::DatabaseConnectionTrait;
use crate::users::entities::users::{Entity as User, Model as UserModel};
use async_trait::async_trait;
use chrono::{Duration, Utc};
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseTransaction, EntityTrait, QueryFilter, Set};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct RefreshTokenRepository {
	db_connection: Arc<dyn DatabaseConnectionTrait>,
}

impl RefreshTokenRepository {
	pub fn new(db_connection: Arc<dyn DatabaseConnectionTrait>) -> Self {
		Self { db_connection }
	}
}

#[async_trait]
pub trait RefreshTokenRepositoryTrait: Send + Sync {
	fn get_db(&self) -> &sea_orm::DatabaseConnection;
	async fn create(&self, user_id: i32, expires_in: i64) -> Result<(RefreshTokenModel, String), AppError>;
	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		expires_in: i64,
	) -> Result<(RefreshTokenModel, String), AppError>;
	async fn find_by_token_and_user_id(&self, token: &str, user_id: i32) -> Result<RefreshTokenModel, AppError>;
	async fn delete_by_token_and_user_id(&self, token: &str, user_id: i32) -> Result<(), AppError>;
	async fn delete_all_by_user_id(&self, user_id: i32) -> Result<(), AppError>;
	async fn delete_expired(&self) -> Result<(), AppError>;
	async fn is_token_valid(&self, token_model: &RefreshTokenModel) -> bool;
	async fn find_user_by_id(&self, user_id: i32) -> Result<UserModel, AppError>;
}

#[async_trait]
impl RefreshTokenRepositoryTrait for RefreshTokenRepository {
	fn get_db(&self) -> &sea_orm::DatabaseConnection {
		self.db_connection.get_connection()
	}

	async fn create(&self, user_id: i32, expires_in: i64) -> Result<(RefreshTokenModel, String), AppError> {
		let token = Uuid::new_v4().to_string();
		let now = Utc::now();
		let expires_at = now + Duration::seconds(expires_in);

		let expires_at_db: sea_orm::prelude::DateTimeWithTimeZone = expires_at.into();
		let now_db: sea_orm::prelude::DateTimeWithTimeZone = now.into();

		let refresh_token = RefreshTokenActiveModel {
			id: ActiveValue::NotSet,
			token: Set(token.clone()),
			expires_at: Set(expires_at_db),
			created_at: Set(now_db),
			updated_at: Set(Some(now_db)),
			user_id: Set(user_id),
		};

		let model = refresh_token.insert(self.get_db()).await.map_err(|err| {
			eprintln!("Error inserting refresh token: {}", err);
			AppError::InternalError
		})?;

		Ok((model, token))
	}

	async fn create_in_transaction(
		&self,
		transaction: &DatabaseTransaction,
		user_id: i32,
		expires_in: i64,
	) -> Result<(RefreshTokenModel, String), AppError> {
		let token = Uuid::new_v4().to_string();
		let now = Utc::now();
		let expires_at = now + Duration::seconds(expires_in);

		let expires_at_db: sea_orm::prelude::DateTimeWithTimeZone = expires_at.into();
		let now_db: sea_orm::prelude::DateTimeWithTimeZone = now.into();

		let refresh_token = RefreshTokenActiveModel {
			id: ActiveValue::NotSet,
			token: Set(token.clone()),
			expires_at: Set(expires_at_db),
			created_at: Set(now_db),
			updated_at: Set(Some(now_db)),
			user_id: Set(user_id),
		};

		let model = refresh_token.insert(transaction).await.map_err(|err| {
			eprintln!("Error inserting refresh token: {}", err);
			AppError::InternalError
		})?;

		Ok((model, token))
	}

	async fn find_by_token_and_user_id(&self, token: &str, user_id: i32) -> Result<RefreshTokenModel, AppError> {
		let refresh_token = RefreshToken::find()
			.filter(refresh_tokens::Column::Token.eq(token))
			.filter(refresh_tokens::Column::UserId.eq(user_id))
			.one(self.get_db())
			.await
			.map_err(|_| AppError::InternalError)?
			.ok_or(AppError::NotFound)?;

		Ok(refresh_token)
	}

	async fn delete_by_token_and_user_id(&self, token: &str, user_id: i32) -> Result<(), AppError> {
		RefreshToken::delete_many()
			.filter(refresh_tokens::Column::Token.eq(token))
			.filter(refresh_tokens::Column::UserId.eq(user_id))
			.exec(self.get_db())
			.await
			.map_err(|_| AppError::InternalError)?;

		Ok(())
	}

	async fn delete_all_by_user_id(&self, user_id: i32) -> Result<(), AppError> {
		RefreshToken::delete_many()
			.filter(refresh_tokens::Column::UserId.eq(user_id))
			.exec(self.get_db())
			.await
			.map_err(|_| AppError::InternalError)?;

		Ok(())
	}

	async fn delete_expired(&self) -> Result<(), AppError> {
		let now = Utc::now();
		let now_db: sea_orm::prelude::DateTimeWithTimeZone = now.into();

		RefreshToken::delete_many()
			.filter(refresh_tokens::Column::ExpiresAt.lt(now_db))
			.exec(self.get_db())
			.await
			.map_err(|_| AppError::InternalError)?;

		Ok(())
	}

	async fn is_token_valid(&self, token_model: &RefreshTokenModel) -> bool {
		let now: sea_orm::prelude::DateTimeWithTimeZone = Utc::now().into();
		token_model.expires_at >= now
	}

	async fn find_user_by_id(&self, user_id: i32) -> Result<UserModel, AppError> {
		User::find_by_id(user_id)
			.one(self.get_db())
			.await
			.map_err(|_| AppError::InternalError)?
			.ok_or_else(|| AppError::NotFound)
	}
}
