use sqlx::PgPool;

use crate::common::error::AppError;
use crate::users::dto::create_user::CreateUserDto;
use crate::users::dto::update_user::UpdateUserDto;
use crate::users::entities::user::User;

#[derive(Clone)]
pub struct UsersRepository {
	db_pool: PgPool,
}

impl UsersRepository {
	pub fn new(db_pool: PgPool) -> Self {
		Self { db_pool }
	}

	pub async fn find_all(&self) -> Result<Vec<User>, AppError> {
		let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC")
			.fetch_all(&self.db_pool)
			.await?;

		Ok(users)
	}

	pub async fn find_by_id(&self, id: i32) -> Result<User, AppError> {
		let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
			.bind(id)
			.fetch_optional(&self.db_pool)
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	pub async fn find_by_username(&self, username: &str) -> Result<User, AppError> {
		let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
			.bind(username)
			.fetch_optional(&self.db_pool)
			.await?
			.ok_or(AppError::NotFound)?;

		Ok(user)
	}

	pub async fn create(&self, dto: CreateUserDto, password_hash: String) -> Result<User, AppError> {
		let user = sqlx::query_as::<_, User>(
			"INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *",
		)
		.bind(&dto.username)
		.bind(&dto.email)
		.bind(&password_hash)
		.fetch_one(&self.db_pool)
		.await?;

		Ok(user)
	}

	pub async fn update(&self, id: i32, dto: UpdateUserDto) -> Result<User, AppError> {
		// Zakładam, że UpdateUserDto nie zawiera hasła, ale jeśli zawiera,
		// to trzeba zmodyfikować tę metodę, aby obsługiwać aktualizację hasła
		let user = sqlx::query_as::<_, User>(
			"UPDATE users SET username = $1, email = $2, updated_at = NOW() WHERE id = $3 RETURNING *",
		)
		.bind(&dto.username)
		.bind(&dto.email)
		.bind(id)
		.fetch_one(&self.db_pool)
		.await?;

		Ok(user)
	}

	pub async fn delete(&self, id: i32) -> Result<(), AppError> {
		sqlx::query("DELETE FROM users WHERE id = $1")
			.bind(id)
			.execute(&self.db_pool)
			.await?;

		Ok(())
	}
}
