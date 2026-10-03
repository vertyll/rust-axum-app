//! In-memory adapters for the identity ports, so use cases run without
//! Keycloak or a database.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use jiff::{SignedDuration, Timestamp};

use crate::identity::application::IdentityService;
use crate::identity::application::ports::{IdentityPorts, KeycloakClient, TokenVerifier};
use crate::identity::application::session::{TokenPair, VerifiedToken};
use crate::identity::domain::{
	Email, IdentityError, KeycloakId, KeycloakIdentity, NewUser, RoleName, User, UserId, UserRepository,
};

pub(crate) fn identity(roles: Vec<RoleName>) -> KeycloakIdentity {
	KeycloakIdentity {
		keycloak_id: KeycloakId::new("7d1c9a52-3a43-4c34-9a8f-2f5d7c6f1b10"),
		email: Email::parse("ada@rust-axum-app.local").unwrap(),
		first_name: "Ada".into(),
		last_name: "Lovelace".into(),
		roles,
	}
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryUsers {
	users: Arc<Mutex<Vec<User>>>,
	next_id: Arc<AtomicI64>,
	failing: Arc<AtomicBool>,
}

impl InMemoryUsers {
	pub(crate) fn fail_writes(&self) {
		self.failing.store(true, Ordering::SeqCst);
	}

	fn check(&self) -> Result<(), IdentityError> {
		if self.failing.load(Ordering::SeqCst) {
			return Err(IdentityError::PersistenceFailure("database down".into()));
		}
		Ok(())
	}
}

impl UserRepository for InMemoryUsers {
	async fn create(&self, user: NewUser) -> Result<User, IdentityError> {
		self.check()?;
		let created = User {
			id: UserId(self.next_id.fetch_add(1, Ordering::SeqCst) + 1),
			keycloak_id: user.keycloak_id,
			email: user.email,
			first_name: user.first_name,
			last_name: user.last_name,
			roles: user.roles,
			created_at: Timestamp::UNIX_EPOCH,
			updated_at: Timestamp::UNIX_EPOCH,
		};
		self.users.lock().unwrap().push(created.clone());
		Ok(created)
	}

	async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
		Ok(self.users.lock().unwrap().iter().find(|user| user.id == id).cloned())
	}

	async fn find_by_keycloak_id(&self, keycloak_id: &KeycloakId) -> Result<Option<User>, IdentityError> {
		Ok(self
			.users
			.lock()
			.unwrap()
			.iter()
			.find(|user| &user.keycloak_id == keycloak_id)
			.cloned())
	}

	async fn list(&self) -> Result<Vec<User>, IdentityError> {
		Ok(self.users.lock().unwrap().clone())
	}

	async fn update(&self, user: &User) -> Result<(), IdentityError> {
		self.check()?;
		let mut users = self.users.lock().unwrap();
		if let Some(stored) = users.iter_mut().find(|stored| stored.id == user.id) {
			*stored = user.clone();
		}
		Ok(())
	}
}

#[derive(Clone, Default)]
pub(crate) struct FakeKeycloak {
	issued: Arc<AtomicI64>,
	revoked: Arc<Mutex<Vec<String>>>,
}

impl FakeKeycloak {
	pub(crate) fn revoked(&self) -> Vec<String> {
		self.revoked.lock().unwrap().clone()
	}

	fn issue(&self) -> TokenPair {
		let n = self.issued.fetch_add(1, Ordering::SeqCst) + 1;
		TokenPair {
			access_token: "access".into(),
			refresh_token: format!("refresh-{n}"),
		}
	}
}

impl KeycloakClient for FakeKeycloak {
	async fn exchange(&self, _code: &str, _code_verifier: &str) -> Result<TokenPair, IdentityError> {
		Ok(self.issue())
	}

	async fn refresh(&self, _refresh_token: &str) -> Result<TokenPair, IdentityError> {
		Ok(self.issue())
	}

	async fn revoke(&self, refresh_token: &str) {
		self.revoked.lock().unwrap().push(refresh_token.to_string());
	}
}

#[derive(Clone)]
pub(crate) struct FakeVerifier {
	identity: Arc<Mutex<KeycloakIdentity>>,
	rejecting: Arc<AtomicBool>,
}

impl Default for FakeVerifier {
	fn default() -> Self {
		Self {
			identity: Arc::new(Mutex::new(identity(vec![RoleName::User]))),
			rejecting: Arc::default(),
		}
	}
}

impl FakeVerifier {
	pub(crate) fn issue(&self, identity: KeycloakIdentity) {
		*self.identity.lock().unwrap() = identity;
	}

	pub(crate) fn reject_all(&self) {
		self.rejecting.store(true, Ordering::SeqCst);
	}
}

impl TokenVerifier for FakeVerifier {
	async fn verify(&self, _token: &str) -> Result<VerifiedToken, IdentityError> {
		if self.rejecting.load(Ordering::SeqCst) {
			return Err(IdentityError::InvalidToken);
		}
		Ok(VerifiedToken {
			identity: self.identity.lock().unwrap().clone(),
			expires_at: Timestamp::now() + SignedDuration::from_secs(300),
		})
	}
}

#[derive(Clone)]
pub(crate) struct TestPorts;

impl IdentityPorts for TestPorts {
	type Users = InMemoryUsers;
	type Keycloak = FakeKeycloak;
	type Verifier = FakeVerifier;
}

pub(crate) struct Harness {
	pub(crate) service: IdentityService<TestPorts>,
	pub(crate) users: InMemoryUsers,
	pub(crate) keycloak: FakeKeycloak,
	pub(crate) verifier: FakeVerifier,
}

impl Harness {
	pub(crate) fn new() -> Self {
		let users = InMemoryUsers::default();
		let keycloak = FakeKeycloak::default();
		let verifier = FakeVerifier::default();
		Self {
			service: IdentityService::new(users.clone(), keycloak.clone(), verifier.clone()),
			users,
			keycloak,
			verifier,
		}
	}
}
