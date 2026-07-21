//! The `User` aggregate: identity business rules as methods, pure by
//! construction — `now` is always an argument and crypto stays behind
//! application ports, so every rule tests without a DB, clock, or keys.
//! construction — `now` is always an argument and crypto stays behind
//! application ports, so every rule tests without a DB, clock, or keys.
//!
//! All identity business rules live here as methods on the aggregate:
//! confirming an e-mail, starting/completing a password reset or e-mail
//! change, deactivating an account. The methods are pure — the current time
//! is always passed in as an argument, and cryptographic concerns (hashing,
//! JWT signatures) stay behind application-layer ports. That keeps every
//! rule unit-testable without a database, a clock or a key.

use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;

use super::error::IdentityError;
use super::role::RoleName;

// Value objects

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub i64);

impl fmt::Display for UserId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		self.0.fmt(f)
	}
}

/// A syntactically valid e-mail address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Email(String);

impl Email {
	pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
		let value = value.into();
		// Structural minimum only; full validation happens at the HTTP edge.
		let mut parts = value.splitn(2, '@');
		match (parts.next(), parts.next()) {
			(Some(local), Some(host)) if !local.is_empty() && host.contains('.') => Ok(Self(value)),
			_ => Err(IdentityError::InvalidEmail),
		}
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for Email {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

/// A username of at least three characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Username(String);

impl Username {
	pub const MIN_LENGTH: usize = 3;

	pub fn parse(value: impl Into<String>) -> Result<Self, IdentityError> {
		let value = value.into();
		if value.chars().count() < Self::MIN_LENGTH {
			return Err(IdentityError::UsernameTooShort);
		}
		Ok(Self(value))
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

impl fmt::Display for Username {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

/// An already-hashed password. The domain never sees plaintext passwords;
/// hashing and verification go through the `PasswordHasher` port.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordHash(String);

impl PasswordHash {
	pub fn new(hash: impl Into<String>) -> Self {
		Self(hash.into())
	}

	pub fn as_str(&self) -> &str {
		&self.0
	}
}

/// A single-use token paired with its expiry — "token without expiry" is
/// unrepresentable (the DB stores two nullable columns).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredToken {
	pub value: String,
	pub expires_at: Timestamp,
}

impl StoredToken {
	pub fn new(value: impl Into<String>, expires_at: Timestamp) -> Self {
		Self { value: value.into(), expires_at }
	}

	fn verify(&self, presented: &str, now: Timestamp) -> Result<(), IdentityError> {
		if self.value != presented {
			return Err(IdentityError::InvalidToken);
		}
		if now > self.expires_at {
			return Err(IdentityError::ExpiredToken);
		}
		Ok(())
	}
}

/// A pending e-mail change: the confirmation token plus the address that
/// will become active once the token is confirmed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingEmailChange {
	pub token: StoredToken,
	pub new_email: Email,
}

// Aggregate

/// A user account that has not been persisted yet (no identity assigned).
#[derive(Debug, Clone)]
pub struct NewUser {
	pub username: Username,
	pub email: Email,
	pub password_hash: PasswordHash,
	pub roles: Vec<RoleName>,
}

impl NewUser {
	/// Registers a new account with the default role.
	pub fn register(username: Username, email: Email, password_hash: PasswordHash) -> Self {
		Self { username, email, password_hash, roles: vec![RoleName::User] }
	}
}

#[derive(Debug, Clone)]
pub struct User {
	pub id: UserId,
	pub username: Username,
	pub email: Email,
	pub password_hash: PasswordHash,
	pub is_email_confirmed: bool,
	pub is_active: bool,
	pub email_confirmation: Option<StoredToken>,
	pub password_reset: Option<StoredToken>,
	pub email_change: Option<PendingEmailChange>,
	pub roles: Vec<RoleName>,
	pub created_at: Timestamp,
	pub updated_at: Timestamp,
}

impl User {

	/// An account may authenticate only when active and confirmed.
	pub fn ensure_can_authenticate(&self) -> Result<(), IdentityError> {
		if !self.is_active {
			return Err(IdentityError::AccountInactive);
		}
		if !self.is_email_confirmed {
			return Err(IdentityError::EmailNotConfirmed);
		}
		Ok(())
	}

	pub fn has_role(&self, role: RoleName) -> bool {
		self.roles.contains(&role)
	}


	pub fn start_email_confirmation(&mut self, token: StoredToken) {
		self.email_confirmation = Some(token);
	}

	pub fn confirm_email(&mut self, presented: &str, now: Timestamp) -> Result<(), IdentityError> {
		if self.is_email_confirmed {
			return Err(IdentityError::EmailAlreadyConfirmed);
		}
		self.email_confirmation
			.as_ref()
			.ok_or(IdentityError::InvalidToken)?
			.verify(presented, now)?;

		self.is_email_confirmed = true;
		self.email_confirmation = None;
		Ok(())
	}


	pub fn start_password_reset(&mut self, token: StoredToken) {
		self.password_reset = Some(token);
	}

	pub fn complete_password_reset(
		&mut self,
		presented: &str,
		new_hash: PasswordHash,
		now: Timestamp,
	) -> Result<(), IdentityError> {
		self.password_reset
			.as_ref()
			.ok_or(IdentityError::InvalidToken)?
			.verify(presented, now)?;

		self.password_hash = new_hash;
		self.password_reset = None;
		Ok(())
	}

	/// Direct password change; the application layer verifies the current
	/// password through the hasher port before calling this.
	pub fn set_password(&mut self, new_hash: PasswordHash) {
		self.password_hash = new_hash;
	}


	pub fn start_email_change(
		&mut self,
		new_email: Email,
		token: StoredToken,
	) -> Result<(), IdentityError> {
		if new_email == self.email {
			return Err(IdentityError::SameEmailAsCurrent);
		}
		self.email_change = Some(PendingEmailChange { token, new_email });
		Ok(())
	}

	/// Confirms a pending e-mail change; returns the previous address so the
	/// caller can record it in the change history.
	pub fn complete_email_change(
		&mut self,
		presented: &str,
		now: Timestamp,
	) -> Result<Email, IdentityError> {
		let pending = self.email_change.as_ref().ok_or(IdentityError::InvalidToken)?;
		pending.token.verify(presented, now)?;

		let previous = std::mem::replace(&mut self.email, pending.new_email.clone());
		self.email_change = None;
		Ok(previous)
	}


	/// "Deleting" a user is a business-level deactivation, never a row delete.
	pub fn deactivate(&mut self) {
		self.is_active = false;
	}
}
#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use super::*;
	use crate::identity::domain::role::RoleName;

	fn user() -> User {
		User {
			id: UserId(1),
			username: Username::parse("alice").unwrap(),
			email: Email::parse("alice@example.com").unwrap(),
			password_hash: PasswordHash::new("hash"),
			is_email_confirmed: false,
			is_active: true,
			email_confirmation: None,
			password_reset: None,
			email_change: None,
			roles: vec![RoleName::User],
			created_at: Timestamp::UNIX_EPOCH,
			updated_at: Timestamp::UNIX_EPOCH,
		}
	}

	fn valid(value: &str) -> StoredToken {
		StoredToken::new(value, Timestamp::MAX)
	}

	fn expired(value: &str) -> StoredToken {
		StoredToken::new(value, Timestamp::UNIX_EPOCH)
	}

	#[test]
	fn email_requires_local_part_and_dotted_host() {
		assert!(Email::parse("alice@example.com").is_ok());
		for bad in ["", "alice", "@example.com", "alice@localhost"] {
			assert!(matches!(Email::parse(bad), Err(IdentityError::InvalidEmail)), "{bad}");
		}
	}

	#[test]
	fn username_enforces_minimum_length() {
		assert!(Username::parse("abc").is_ok());
		assert!(matches!(Username::parse("ab"), Err(IdentityError::UsernameTooShort)));
	}

	#[test]
	fn register_assigns_default_role() {
		let new = NewUser::register(
			Username::parse("alice").unwrap(),
			Email::parse("alice@example.com").unwrap(),
			PasswordHash::new("hash"),
		);
		assert_eq!(new.roles, vec![RoleName::User]);
	}

	#[test]
	fn confirm_email_happy_path_clears_token() {
		let mut user = user();
		user.start_email_confirmation(valid("token"));
		user.confirm_email("token", Timestamp::now()).unwrap();
		assert!(user.is_email_confirmed);
		assert!(user.email_confirmation.is_none());
	}

	#[test]
	fn confirm_email_rejects_wrong_expired_and_repeated() {
		let mut user = user();
		user.start_email_confirmation(valid("token"));
		let wrong = user.confirm_email("other", Timestamp::now()).unwrap_err();
		assert!(matches!(wrong, IdentityError::InvalidToken));

		user.email_confirmation = Some(expired("token"));
		let late = user.confirm_email("token", Timestamp::now()).unwrap_err();
		assert!(matches!(late, IdentityError::ExpiredToken));

		user.is_email_confirmed = true;
		let again = user.confirm_email("token", Timestamp::now()).unwrap_err();
		assert!(matches!(again, IdentityError::EmailAlreadyConfirmed));
	}

	#[test]
	fn password_reset_replaces_hash_once() {
		let mut user = user();
		user.start_password_reset(valid("reset"));
		user.complete_password_reset("reset", PasswordHash::new("new"), Timestamp::now())
			.unwrap();
		assert_eq!(user.password_hash, PasswordHash::new("new"));
		assert!(user.password_reset.is_none());

		let repeat = user
			.complete_password_reset("reset", PasswordHash::new("newer"), Timestamp::now())
			.unwrap_err();
		assert!(matches!(repeat, IdentityError::InvalidToken));
	}

	#[test]
	fn email_change_swaps_address_and_returns_previous() {
		let mut user = user();
		let new_email = Email::parse("new@example.com").unwrap();
		user.start_email_change(new_email.clone(), valid("change")).unwrap();

		let previous = user.complete_email_change("change", Timestamp::now()).unwrap();
		assert_eq!(previous.as_str(), "alice@example.com");
		assert_eq!(user.email, new_email);
		assert!(user.email_change.is_none());
	}

	#[test]
	fn email_change_rejects_current_address() {
		let mut user = user();
		let same = user.email.clone();
		let err = user.start_email_change(same, valid("change")).unwrap_err();
		assert!(matches!(err, IdentityError::SameEmailAsCurrent));
	}

	#[test]
	fn authentication_requires_active_confirmed_account() {
		let mut user = user();
		assert!(matches!(
			user.ensure_can_authenticate(),
			Err(IdentityError::EmailNotConfirmed)
		));

		user.is_email_confirmed = true;
		assert!(user.ensure_can_authenticate().is_ok());

		user.deactivate();
		assert!(matches!(user.ensure_can_authenticate(), Err(IdentityError::AccountInactive)));
	}
}
