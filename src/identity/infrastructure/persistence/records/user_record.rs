use jiff::Timestamp;
use toasty::Deferred;

use super::UserRoleRecord;
use crate::identity::domain::{Email, IdentityError, KeycloakId, RoleName, User, UserId};

#[derive(Debug, toasty::Model)]
#[table = "users"]
pub struct UserRecord {
	#[key]
	#[auto]
	pub id: i64,
	#[unique]
	pub keycloak_id: String,
	#[unique]
	pub email: String,
	pub first_name: String,
	pub last_name: String,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[has_many]
	pub user_roles: Deferred<Vec<UserRoleRecord>>,
}

impl UserRecord {
	pub fn to_domain(&self, roles: Vec<RoleName>) -> Result<User, IdentityError> {
		Ok(User {
			id: UserId(self.id),
			keycloak_id: KeycloakId::new(self.keycloak_id.clone()),
			email: Email::parse(self.email.clone())
				.map_err(|_| IdentityError::CorruptData(format!("user {}: invalid e-mail", self.id)))?,
			first_name: self.first_name.clone(),
			last_name: self.last_name.clone(),
			roles,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}
