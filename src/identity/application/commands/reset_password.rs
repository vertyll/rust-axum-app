#[derive(Debug, Clone)]
pub struct ResetPassword {
	pub token: String,
	pub new_password: String,
}
