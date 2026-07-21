use jiff::Timestamp;
use toasty::Deferred;

use super::{RoleRecord, UserRecord};

#[derive(Debug, toasty::Model)]
#[table = "user_roles"]
pub struct UserRoleRecord {
	#[key]
	#[auto]
	pub id: i64,
	#[index]
	pub user_id: i64,
	#[index]
	pub role_id: i64,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[belongs_to(key = user_id, references = id)]
	pub user_record: Deferred<UserRecord>,
	#[belongs_to(key = role_id, references = id)]
	pub role_record: Deferred<RoleRecord>,
}
