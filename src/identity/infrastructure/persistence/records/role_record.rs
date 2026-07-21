use jiff::Timestamp;
use toasty::Deferred;

use super::UserRoleRecord;

#[derive(Debug, toasty::Model)]
#[table = "roles"]
pub struct RoleRecord {
	#[key]
	#[auto]
	pub id: i64,
	#[unique]
	pub name: String,
	pub description: Option<String>,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[has_many]
	pub user_roles: Deferred<Vec<UserRoleRecord>>,
}
