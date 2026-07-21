use jiff::{SignedDuration, Timestamp};

use super::user::UserId;

/// A refresh-token session — its own aggregate with its own repository and
/// transactional boundary (per-device logout, expiry sweeps).
#[derive(Debug, Clone)]
pub struct RefreshSession {
	pub token: String,
	pub user_id: UserId,
	pub expires_at: Timestamp,
}

impl RefreshSession {
	/// The token value comes from the caller (the application layer owns
	/// randomness), keeping this constructor deterministic.
	pub fn issue(user_id: UserId, token: String, ttl_seconds: i64, now: Timestamp) -> Self {
		Self {
			token,
			user_id,
			expires_at: now
				.checked_add(SignedDuration::from_secs(ttl_seconds))
				.unwrap_or(Timestamp::MAX),
		}
	}

	pub fn is_expired(&self, now: Timestamp) -> bool {
		now > self.expires_at
	}
}
#[cfg(test)]
mod tests {
	use jiff::{SignedDuration, Timestamp};

	use super::RefreshSession;
	use crate::identity::domain::user::UserId;

	#[test]
	fn expires_after_ttl() {
		let now = Timestamp::UNIX_EPOCH;
		let session = RefreshSession::issue(UserId(1), "token".into(), 60, now);
		assert!(!session.is_expired(now));

		let later = now.checked_add(SignedDuration::from_secs(61)).unwrap();
		assert!(session.is_expired(later));
	}

	#[test]
	fn saturates_on_absurd_ttl() {
		let session = RefreshSession::issue(UserId(1), "token".into(), i64::MAX, Timestamp::now());
		assert_eq!(session.expires_at, Timestamp::MAX);
	}
}
