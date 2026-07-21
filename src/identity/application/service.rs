//! The application service the use cases attach to. One façade type, wired
//! once in the composition root; every use case lives in `use_cases/` as
//! its own `impl` block, so this file holds only state and shared helpers.

use jiff::{SignedDuration, Timestamp};
use uuid::Uuid;

use super::commands::RegisterUser;
use super::ports::{IdentityMailer, IdentityPorts, PasswordHasher, TokenService};
use super::token::{AuthTokens, TokenKind};
use crate::identity::domain::{
	IdentityError, NewUser, RefreshSession, SessionRepository, StoredToken, User, UserRepository,
};

/// Fields are typed through the [`IdentityPorts`] type family, so there is
/// exactly one generic parameter; `pub(super)` keeps them reachable from
/// the sibling `use_cases` files and invisible outside the layer.
#[derive(Clone)]
pub struct IdentityService<P: IdentityPorts> {
	pub(super) users: P::Users,
	pub(super) sessions: P::Sessions,
	pub(super) hasher: P::Hasher,
	pub(super) tokens: P::Tokens,
	pub(super) mailer: P::Mailer,
	pub(super) refresh_ttl_seconds: i64,
}

impl<P: IdentityPorts> IdentityService<P> {
	pub fn new(
		users: P::Users,
		sessions: P::Sessions,
		hasher: P::Hasher,
		tokens: P::Tokens,
		mailer: P::Mailer,
		refresh_ttl_seconds: i64,
	) -> Self {
		Self {
			users,
			sessions,
			hasher,
			tokens,
			mailer,
			refresh_ttl_seconds,
		}
	}

	pub fn refresh_ttl_seconds(&self) -> i64 {
		self.refresh_ttl_seconds
	}

	pub(super) async fn open_session(&self, user: &User) -> Result<AuthTokens, IdentityError> {
		let session = RefreshSession::issue(
			user.id,
			Uuid::new_v4().to_string(),
			self.refresh_ttl_seconds,
			Timestamp::now(),
		);
		self.sessions.create(&session).await?;

		Ok(AuthTokens {
			access_token: self.tokens.sign_access(user)?,
			refresh_token: session.token,
		})
	}

	pub(super) fn confirmation_token(&self, value: &str) -> StoredToken {
		// jiff arithmetic is fallible only for calendar spans; an absolute
		// duration saturates at the range end, like `RefreshSession::issue`.
		let ttl = SignedDuration::from_secs(self.tokens.confirmation_ttl_seconds());
		let expires_at = Timestamp::now().checked_add(ttl).unwrap_or(Timestamp::MAX);
		StoredToken::new(value, expires_at)
	}

	/// Shared by the `register` and `create_user` use cases.
	pub(super) async fn create_account(&self, cmd: RegisterUser) -> Result<User, IdentityError> {
		if self.users.find_by_email(&cmd.email).await?.is_some() {
			return Err(IdentityError::EmailTaken);
		}
		if self.users.find_by_username(&cmd.username).await?.is_some() {
			return Err(IdentityError::UsernameTaken);
		}

		let password_hash = self.hasher.hash(cmd.password).await?;
		let mut user = self
			.users
			.create(NewUser::register(cmd.username, cmd.email, password_hash))
			.await?;

		let token = self
			.tokens
			.sign_confirmation(TokenKind::EmailConfirmation, user.id, &user.email, None)?;
		user.start_email_confirmation(self.confirmation_token(&token));
		self.users.update(&user).await?;

		// E-mail after commit, best-effort: rolling registration back on an
		// SMTP failure would couple a network call into persistence (the
		// production-grade fix is an outbox).
		if let Err(err) = self
			.mailer
			.send_email_confirmation(&user.email, &user.username, &token)
			.await
		{
			tracing::error!(user_id = %user.id, "failed to send confirmation e-mail: {err}");
		}

		Ok(user)
	}
}
