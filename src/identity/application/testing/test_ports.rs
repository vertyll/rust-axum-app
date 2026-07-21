use super::fake_hasher::FakeHasher;
use super::fake_mailer::FakeMailer;
use super::fake_tokens::FakeTokens;
use super::in_memory_sessions::InMemorySessions;
use super::in_memory_users::InMemoryUsers;
use crate::identity::application::ports::IdentityPorts;

#[derive(Clone)]
pub(crate) struct TestPorts;

impl IdentityPorts for TestPorts {
	type Users = InMemoryUsers;
	type Sessions = InMemorySessions;
	type Hasher = FakeHasher;
	type Tokens = FakeTokens;
	type Mailer = FakeMailer;
}
