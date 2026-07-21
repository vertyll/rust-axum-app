#[derive(Debug, Clone)]
pub struct ChangePassword {
	pub current_password: String,
	pub new_password: String,
}
