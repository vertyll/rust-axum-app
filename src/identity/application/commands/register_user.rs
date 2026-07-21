use crate::identity::domain::{Email, Username};

#[derive(Debug, Clone)]
pub struct RegisterUser {
	pub username: Username,
	pub email: Email,
	pub password: String,
}
