use super::{IdentityMailer, PasswordHasher, TokenService};
use crate::identity::domain::{SessionRepository, UserRepository};

pub trait IdentityPorts: Clone + Send + Sync + 'static {
	type Users: UserRepository;
	type Sessions: SessionRepository;
	type Hasher: PasswordHasher;
	type Tokens: TokenService;
	type Mailer: IdentityMailer;
}
