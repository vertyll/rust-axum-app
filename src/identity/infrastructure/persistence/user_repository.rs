use std::collections::HashMap;

use super::records::{EmailHistoryRecord, RoleRecord, UserRecord, UserRoleRecord};
use crate::identity::domain::{
	Email, IdentityError, NewUser, RoleName, User, UserId, UserRepository, Username,
};

#[derive(Clone)]
pub struct ToastyUserRepository {
	db: toasty::Db,
}

impl ToastyUserRepository {
	pub fn new(db: toasty::Db) -> Self {
		Self { db }
	}

	async fn load_roles(
		db: &mut toasty::Db,
		user_id: i64,
	) -> Result<Vec<RoleName>, IdentityError> {
		let roles = RoleRecord::filter(
			RoleRecord::fields()
				.user_roles()
				.any(UserRoleRecord::fields().user_id().eq(user_id)),
		)
		.exec(db)
		.await
		.map_err(persistence)?;

		Ok(roles.iter().filter_map(|role| role.name.parse().ok()).collect())
	}

	async fn map_one(
		db: &mut toasty::Db,
		record: Option<UserRecord>,
	) -> Result<Option<User>, IdentityError> {
		match record {
			Some(record) => {
				let roles = Self::load_roles(db, record.id).await?;
				record.to_domain(roles).map(Some)
			}
			None => Ok(None),
		}
	}
}

impl UserRepository for ToastyUserRepository {
	async fn create(&self, user: NewUser) -> Result<User, IdentityError> {
		let mut db = self.db.clone();
		let mut tx = db.transaction().await.map_err(persistence)?;

		let record = toasty::create!(UserRecord {
			username: user.username.as_str(),
			email: user.email.as_str(),
			password_hash: user.password_hash.as_str(),
		})
		.exec(&mut tx)
		.await
		.map_err(persistence)?;

		for role in &user.roles {
			let role_record = RoleRecord::filter_by_name(role.as_str())
				.first()
				.exec(&mut tx)
				.await
				.map_err(persistence)?
				.ok_or_else(|| {
					IdentityError::PersistenceFailure(format!("role '{role}' is not seeded"))
				})?;

			toasty::create!(UserRoleRecord { user_id: record.id, role_id: role_record.id })
				.exec(&mut tx)
				.await
				.map_err(persistence)?;
		}

		tx.commit().await.map_err(persistence)?;

		record.to_domain(user.roles)
	}

	async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
		let mut db = self.db.clone();
		let record = UserRecord::filter_by_id(id.0)
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;
		Self::map_one(&mut db, record).await
	}

	async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError> {
		let mut db = self.db.clone();
		let record = UserRecord::filter_by_email(email.as_str())
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;
		Self::map_one(&mut db, record).await
	}

	async fn find_by_username(&self, username: &Username) -> Result<Option<User>, IdentityError> {
		let mut db = self.db.clone();
		let record = UserRecord::filter_by_username(username.as_str())
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;
		Self::map_one(&mut db, record).await
	}

	async fn list(&self) -> Result<Vec<User>, IdentityError> {
		let mut db = self.db.clone();

		let records = UserRecord::all().exec(&mut db).await.map_err(persistence)?;
		let roles = RoleRecord::all().exec(&mut db).await.map_err(persistence)?;
		let links = UserRoleRecord::all().exec(&mut db).await.map_err(persistence)?;

		let role_names: HashMap<i64, RoleName> = roles
			.iter()
			.filter_map(|role| Some((role.id, role.name.parse().ok()?)))
			.collect();

		let mut roles_by_user: HashMap<i64, Vec<RoleName>> = HashMap::new();
		for link in &links {
			if let Some(role) = role_names.get(&link.role_id) {
				roles_by_user.entry(link.user_id).or_default().push(*role);
			}
		}

		records
			.into_iter()
			.map(|record| {
				let roles = roles_by_user.remove(&record.id).unwrap_or_default();
				record.to_domain(roles)
			})
			.collect()
	}

	async fn update(&self, user: &User) -> Result<(), IdentityError> {
		let mut db = self.db.clone();
		apply_update(&mut db, user).await
	}

	async fn save_email_change(
		&self,
		user: &User,
		previous_email: &Email,
	) -> Result<(), IdentityError> {
		let mut db = self.db.clone();
		let mut tx = db.transaction().await.map_err(persistence)?;

		apply_update(&mut tx, user).await?;

		toasty::create!(EmailHistoryRecord {
			user_id: user.id.0,
			old_email: previous_email.as_str(),
			new_email: user.email.as_str(),
			email_change_at: jiff::Timestamp::now(),
		})
		.exec(&mut tx)
		.await
		.map_err(persistence)?;

		tx.commit().await.map_err(persistence)
	}
}

/// Writes the aggregate's mutable columns. Generic over the executor so the
/// same statement runs standalone or inside `save_email_change`'s transaction.
async fn apply_update<E>(executor: &mut E, user: &User) -> Result<(), IdentityError>
where
	E: toasty::Executor,
{
	let email_change = user.email_change.as_ref();

	toasty::update!(UserRecord::filter_by_id(user.id.0) {
		username: user.username.as_str(),
		email: user.email.as_str(),
		password_hash: user.password_hash.as_str(),
		is_email_confirmed: user.is_email_confirmed,
		is_active: user.is_active,
		email_confirmation_token: user.email_confirmation.as_ref().map(|t| t.value.clone()),
		email_confirmation_token_expiry: user.email_confirmation.as_ref().map(|t| t.expires_at),
		password_reset_token: user.password_reset.as_ref().map(|t| t.value.clone()),
		password_reset_token_expiry: user.password_reset.as_ref().map(|t| t.expires_at),
		email_change_token: email_change.map(|c| c.token.value.clone()),
		email_change_token_expiry: email_change.map(|c| c.token.expires_at),
		pending_email: email_change.map(|c| c.new_email.as_str().to_string()),
	})
	.exec(executor)
	.await
	.map_err(persistence)?;

	Ok(())
}

pub(crate) fn persistence(err: toasty::Error) -> IdentityError {
	IdentityError::PersistenceFailure(err.to_string())
}
