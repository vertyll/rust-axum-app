use std::collections::HashMap;

use super::records::{RoleRecord, UserRecord, UserRoleRecord};
use crate::identity::domain::{IdentityError, KeycloakId, NewUser, RoleName, User, UserId, UserRepository};

#[derive(Clone)]
pub struct ToastyUserRepository {
	db: toasty::Db,
}

impl ToastyUserRepository {
	pub fn new(db: toasty::Db) -> Self {
		Self { db }
	}

	async fn load_roles(db: &mut toasty::Db, user_id: i64) -> Result<Vec<RoleName>, IdentityError> {
		let roles = RoleRecord::filter(
			RoleRecord::fields()
				.user_roles()
				.any(UserRoleRecord::fields().user_id().eq(user_id)),
		)
		.exec(db)
		.await
		.map_err(persistence)?;

		let mut names: Vec<RoleName> = roles.iter().filter_map(|role| role.name.parse().ok()).collect();
		names.sort_by_key(|role| role.as_str());
		Ok(names)
	}

	async fn map_one(db: &mut toasty::Db, record: Option<UserRecord>) -> Result<Option<User>, IdentityError> {
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
			keycloak_id: user.keycloak_id.as_str(),
			email: user.email.as_str(),
			first_name: user.first_name.as_str(),
			last_name: user.last_name.as_str(),
		})
		.exec(&mut tx)
		.await
		.map_err(persistence)?;

		link_roles(&mut tx, record.id, &user.roles).await?;
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

	async fn find_by_keycloak_id(&self, keycloak_id: &KeycloakId) -> Result<Option<User>, IdentityError> {
		let mut db = self.db.clone();
		let record = UserRecord::filter_by_keycloak_id(keycloak_id.as_str())
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
				let mut roles = roles_by_user.remove(&record.id).unwrap_or_default();
				roles.sort_by_key(|role| role.as_str());
				record.to_domain(roles)
			})
			.collect()
	}

	async fn update(&self, user: &User) -> Result<(), IdentityError> {
		let mut db = self.db.clone();
		let mut tx = db.transaction().await.map_err(persistence)?;

		toasty::update!(UserRecord::filter_by_id(user.id.0) {
			email: user.email.as_str(),
			first_name: user.first_name.as_str(),
			last_name: user.last_name.as_str(),
		})
		.exec(&mut tx)
		.await
		.map_err(persistence)?;

		UserRoleRecord::filter_by_user_id(user.id.0)
			.delete()
			.exec(&mut tx)
			.await
			.map_err(persistence)?;
		link_roles(&mut tx, user.id.0, &user.roles).await?;

		tx.commit().await.map_err(persistence)
	}
}

async fn link_roles<E>(executor: &mut E, user_id: i64, roles: &[RoleName]) -> Result<(), IdentityError>
where
	E: toasty::Executor,
{
	for role in roles {
		let role_record = RoleRecord::filter_by_name(role.as_str())
			.first()
			.exec(executor)
			.await
			.map_err(persistence)?
			.ok_or_else(|| IdentityError::PersistenceFailure(format!("role '{role}' is not seeded")))?;

		toasty::create!(UserRoleRecord {
			user_id: user_id,
			role_id: role_record.id
		})
		.exec(executor)
		.await
		.map_err(persistence)?;
	}
	Ok(())
}

pub(crate) fn persistence(err: toasty::Error) -> IdentityError {
	IdentityError::PersistenceFailure(err.to_string())
}
