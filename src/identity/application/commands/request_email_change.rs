use crate::identity::domain::Email;

#[derive(Debug, Clone)]
pub struct RequestEmailChange {
	pub new_email: Email,
}
