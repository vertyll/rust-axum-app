use super::{IdentityMailer, PasswordHasher, TokenService};
use crate::identity::domain::{SessionRepository, UserRepository};

/// Bundles the five driven ports behind one generic parameter as a pure
/// type family: no methods, only associated types. Every use case is
/// `impl<P: IdentityPorts>` instead of dragging `<U, S, H, T, M>` around,
/// the service stores the adapters directly as `P::Users` etc., and adding
/// a port touches this trait plus the composition root — not every
/// use-case file. Dispatch stays fully static.
pub trait IdentityPorts: Clone + Send + Sync + 'static {
	type Users: UserRepository;
	type Sessions: SessionRepository;
	type Hasher: PasswordHasher;
	type Tokens: TokenService;
	type Mailer: IdentityMailer;
}
