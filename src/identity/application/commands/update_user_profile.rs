use crate::identity::domain::{Email, Username};

#[derive(Debug, Clone, Default)]
pub struct UpdateUserProfile {
	pub username: Option<Username>,
	pub email: Option<Email>,
}
