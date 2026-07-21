use jiff::Timestamp;
use toasty::Deferred;

use super::UserRecord;

#[derive(Debug, toasty::Model)]
#[table = "refresh_tokens"]
pub struct RefreshTokenRecord {
	#[key]
	#[auto]
	pub id: i64,
	/// Unique: the refresh lookup must be indexed.
	#[unique]
	pub token: String,
	pub expires_at: Timestamp,
	#[index]
	pub user_id: i64,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[belongs_to(key = user_id, references = id)]
	pub user_record: Deferred<UserRecord>,
}
