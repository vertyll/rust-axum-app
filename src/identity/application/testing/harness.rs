use super::fake_hasher::FakeHasher;
use super::fake_mailer::FakeMailer;
use super::fake_tokens::FakeTokens;
use super::in_memory_sessions::InMemorySessions;
use super::in_memory_users::InMemoryUsers;
use super::test_ports::TestPorts;
use crate::identity::application::commands::RegisterUser;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{Email, User, Username};

/// Everything a use-case test needs: the service plus handles to the fakes
/// behind it (clones share state).
pub(crate) struct Harness {
	pub service: IdentityService<TestPorts>,
	pub users: InMemoryUsers,
	pub sessions: InMemorySessions,
	pub tokens: FakeTokens,
	pub mailer: FakeMailer,
}

pub(crate) fn harness() -> Harness {
	let users = InMemoryUsers::default();
	let sessions = InMemorySessions::default();
	let tokens = FakeTokens::default();
	let mailer = FakeMailer::default();
	let service: IdentityService<TestPorts> = IdentityService::new(
		users.clone(),
		sessions.clone(),
		FakeHasher,
		tokens.clone(),
		mailer.clone(),
		3600,
	);
	Harness { service, users, sessions, tokens, mailer }
}

pub(crate) fn register_cmd(username: &str, email: &str) -> RegisterUser {
	RegisterUser {
		username: Username::parse(username).unwrap(),
		email: Email::parse(email).unwrap(),
		password: "password123".into(),
	}
}

/// Creates an account and marks it e-mail-confirmed, ready to log in with
/// the password from [`register_cmd`].
pub(crate) async fn confirmed_user(h: &Harness, username: &str, email: &str) -> User {
	let created = h.service.create_user(register_cmd(username, email)).await.unwrap();
	let mut user = h.users.get(created.id).unwrap();
	user.is_email_confirmed = true;
	h.users.set(user.clone());
	user
}
