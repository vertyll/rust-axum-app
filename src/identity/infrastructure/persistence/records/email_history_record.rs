use jiff::Timestamp;
use toasty::Deferred;

use super::UserRecord;

#[derive(Debug, toasty::Model)]
#[table = "users_email_history"]
pub struct EmailHistoryRecord {
	#[key]
	#[auto]
	pub id: i64,
	pub old_email: String,
	pub new_email: String,
	pub email_change_at: Timestamp,
	#[index]
	pub user_id: i64,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[belongs_to(key = user_id, references = id)]
	pub user_record: Deferred<UserRecord>,
}
