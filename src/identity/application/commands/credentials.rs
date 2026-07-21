/// Raw login input. The username stays a `String` on purpose: parsing
/// happens inside the `login` use case, so failures collapse into
/// `InvalidCredentials` instead of a validation error revealing the format.
#[derive(Debug, Clone)]
pub struct Credentials {
	pub username: String,
	pub password: String,
}
