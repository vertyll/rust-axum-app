//! In-memory adapters for the identity ports, so use cases run without
//! Keycloak or a database.

use crate::shared_kernel::pagination::{PageRequest, PageResult};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};

use jiff::{SignedDuration, Timestamp};

use crate::identity::application::IdentityService;
use crate::identity::application::permissions::CATALOGUE;
use crate::identity::application::ports::{IdentityPorts, KeycloakClient, RealmAdmin, TokenVerifier};
use crate::identity::application::session::{TokenPair, VerifiedToken};
use crate::identity::domain::{
    Email, IdentityError, KeycloakId, KeycloakIdentity, NewUser, RegisteredPermission, Role, RoleName, RoleRepository,
    User, UserId, UserRepository,
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

    async fn page(&self, request: PageRequest) -> Result<PageResult<User>, IdentityError> {
        Ok(request.slice(self.users.lock().unwrap().clone()))
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
            identity: Arc::new(Mutex::new(identity(vec![RoleName::parse(RoleName::USER).unwrap()]))),
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

#[derive(Clone, Default)]
pub(crate) struct InMemoryRoles {
    roles: Arc<Mutex<BTreeMap<RoleName, Role>>>,
    permissions: Arc<Mutex<Vec<RegisteredPermission>>>,
}

impl RoleRepository for InMemoryRoles {
    async fn list(&self) -> Result<Vec<Role>, IdentityError> {
        Ok(self.roles.lock().unwrap().values().cloned().collect())
    }

    async fn find(&self, name: &RoleName) -> Result<Option<Role>, IdentityError> {
        Ok(self.roles.lock().unwrap().get(name).cloned())
    }

    async fn find_many(&self, names: &[RoleName]) -> Result<Vec<Role>, IdentityError> {
        let roles = self.roles.lock().unwrap();
        Ok(names.iter().filter_map(|name| roles.get(name).cloned()).collect())
    }

    async fn create(&self, role: &Role) -> Result<(), IdentityError> {
        self.roles.lock().unwrap().insert(role.name.clone(), role.clone());
        Ok(())
    }

    async fn update(&self, role: &Role) -> Result<(), IdentityError> {
        self.create(role).await
    }

    async fn delete(&self, name: &RoleName) -> Result<(), IdentityError> {
        self.roles.lock().unwrap().remove(name);
        Ok(())
    }

    async fn permissions(&self) -> Result<Vec<RegisteredPermission>, IdentityError> {
        let mut permissions = self.permissions.lock().unwrap().clone();
        permissions.sort();
        Ok(permissions)
    }

    async fn register(&self, declared: &[RegisteredPermission]) -> Result<(), IdentityError> {
        *self.permissions.lock().unwrap() = declared.to_vec();
        for role in self.roles.lock().unwrap().values_mut() {
            role.permissions
                .retain(|name| declared.iter().any(|permission| &permission.name == name));
        }
        Ok(())
    }
}

/// What the fake realm was asked to do, in order: `create ADMIN`,
/// `assign <account> EDITOR`.
#[derive(Clone, Default)]
pub(crate) struct FakeRealm {
    calls: Arc<Mutex<Vec<String>>>,
    failing: Arc<AtomicBool>,
}

impl FakeRealm {
    pub(crate) fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }

    pub(crate) fn fail(&self) {
        self.failing.store(true, Ordering::SeqCst);
    }

    fn record(&self, call: String) -> Result<(), IdentityError> {
        if self.failing.load(Ordering::SeqCst) {
            return Err(IdentityError::IdentityProviderUnavailable("down".into()));
        }
        self.calls.lock().unwrap().push(call);
        Ok(())
    }
}

impl RealmAdmin for FakeRealm {
    async fn create_role(&self, name: &RoleName, _description: &str) -> Result<(), IdentityError> {
        self.record(format!("create {}", name.realm_role()))
    }

    async fn delete_role(&self, name: &RoleName) -> Result<(), IdentityError> {
        self.record(format!("delete {}", name.realm_role()))
    }

    async fn assign_role(&self, account: &KeycloakId, name: &RoleName) -> Result<(), IdentityError> {
        self.record(format!("assign {account} {}", name.realm_role()))
    }

    async fn remove_role(&self, account: &KeycloakId, name: &RoleName) -> Result<(), IdentityError> {
        self.record(format!("remove {account} {}", name.realm_role()))
    }
}

#[derive(Clone)]
pub(crate) struct TestPorts;

impl IdentityPorts for TestPorts {
    type Users = InMemoryUsers;
    type Roles = InMemoryRoles;
    type Keycloak = FakeKeycloak;
    type Realm = FakeRealm;
    type Verifier = FakeVerifier;
}

pub(crate) struct Harness {
    pub(crate) service: IdentityService<TestPorts>,
    pub(crate) users: InMemoryUsers,
    pub(crate) roles: InMemoryRoles,
    pub(crate) keycloak: FakeKeycloak,
    pub(crate) realm: FakeRealm,
    pub(crate) verifier: FakeVerifier,
}

impl Harness {
    /// A service whose registry holds the identity module's own catalogue.
    pub(crate) async fn new() -> Self {
        let users = InMemoryUsers::default();
        let roles = InMemoryRoles::default();
        let keycloak = FakeKeycloak::default();
        let realm = FakeRealm::default();
        let verifier = FakeVerifier::default();
        let service = IdentityService::new(
            users.clone(),
            roles.clone(),
            keycloak.clone(),
            realm.clone(),
            verifier.clone(),
        );
        service.register(&[CATALOGUE]).await.unwrap();
        Self {
            service,
            users,
            roles,
            keycloak,
            realm,
            verifier,
        }
    }
}
