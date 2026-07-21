use crate::identity::domain::Email;

#[derive(Debug, Clone)]
pub struct RequestPasswordReset {
	pub email: Email,
}
