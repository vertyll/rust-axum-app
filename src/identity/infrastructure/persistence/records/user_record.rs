use jiff::Timestamp;
use toasty::Deferred;

use super::{EmailHistoryRecord, RefreshTokenRecord, UserRoleRecord};
use crate::identity::domain::{
	Email, IdentityError, PasswordHash, PendingEmailChange, RoleName, StoredToken, User, UserId, Username,
};

#[derive(Debug, toasty::Model)]
#[table = "users"]
pub struct UserRecord {
	#[key]
	#[auto]
	pub id: i64,
	#[unique]
	pub username: String,
	#[unique]
	pub email: String,
	pub password_hash: String,
	#[default(false)]
	pub is_email_confirmed: bool,
	#[default(true)]
	pub is_active: bool,
	pub email_confirmation_token: Option<String>,
	pub email_confirmation_token_expiry: Option<Timestamp>,
	pub email_change_token: Option<String>,
	pub email_change_token_expiry: Option<Timestamp>,
	pub password_reset_token: Option<String>,
	pub password_reset_token_expiry: Option<Timestamp>,
	pub pending_email: Option<String>,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,

	#[has_many]
	pub user_roles: Deferred<Vec<UserRoleRecord>>,
	#[has_many]
	pub refresh_tokens: Deferred<Vec<RefreshTokenRecord>>,
	#[has_many]
	pub email_history: Deferred<Vec<EmailHistoryRecord>>,
}

impl UserRecord {
	pub fn to_domain(&self, roles: Vec<RoleName>) -> Result<User, IdentityError> {
		let corrupt = |what: &str| IdentityError::CorruptData(format!("user {}: {what}", self.id));

		let email_confirmation = stored_token(
			self.email_confirmation_token.clone(),
			self.email_confirmation_token_expiry,
		);
		let password_reset = stored_token(self.password_reset_token.clone(), self.password_reset_token_expiry);

		let email_change = match (
			stored_token(self.email_change_token.clone(), self.email_change_token_expiry),
			&self.pending_email,
		) {
			(Some(token), Some(pending)) => Some(PendingEmailChange {
				token,
				new_email: Email::parse(pending.clone()).map_err(|_| corrupt("invalid pending e-mail"))?,
			}),
			_ => None,
		};

		Ok(User {
			id: UserId(self.id),
			username: Username::parse(self.username.clone()).map_err(|_| corrupt("invalid username"))?,
			email: Email::parse(self.email.clone()).map_err(|_| corrupt("invalid e-mail"))?,
			password_hash: PasswordHash::new(self.password_hash.clone()),
			is_email_confirmed: self.is_email_confirmed,
			is_active: self.is_active,
			email_confirmation,
			password_reset,
			email_change,
			roles,
			created_at: self.created_at,
			updated_at: self.updated_at,
		})
	}
}

fn stored_token(value: Option<String>, expires_at: Option<Timestamp>) -> Option<StoredToken> {
	Some(StoredToken {
		value: value?,
		expires_at: expires_at?,
	})
}
