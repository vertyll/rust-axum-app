use std::sync::{Arc, Mutex};

use jiff::Timestamp;

use crate::identity::domain::{
	Email, IdentityError, NewUser, User, UserId, UserRepository, Username,
};

#[derive(Default)]
struct State {
	next_id: i64,
	users: Vec<User>,
	email_history: Vec<(UserId, Email)>,
}

#[derive(Clone, Default)]
pub(crate) struct InMemoryUsers {
	state: Arc<Mutex<State>>,
}

impl InMemoryUsers {
	pub fn get(&self, id: UserId) -> Option<User> {
		self.state.lock().unwrap().users.iter().find(|user| user.id == id).cloned()
	}

	/// Inserts or replaces, for direct test setup.
	pub fn set(&self, user: User) {
		let mut state = self.state.lock().unwrap();
		match state.users.iter().position(|existing| existing.id == user.id) {
			Some(index) => state.users[index] = user,
			None => state.users.push(user),
		}
	}

	pub fn email_history(&self) -> Vec<(UserId, Email)> {
		self.state.lock().unwrap().email_history.clone()
	}
}

impl UserRepository for InMemoryUsers {
	async fn create(&self, user: NewUser) -> Result<User, IdentityError> {
		let mut state = self.state.lock().unwrap();
		state.next_id += 1;
		let now = Timestamp::now();
		let user = User {
			id: UserId(state.next_id),
			username: user.username,
			email: user.email,
			password_hash: user.password_hash,
			is_email_confirmed: false,
			is_active: true,
			email_confirmation: None,
			password_reset: None,
			email_change: None,
			roles: user.roles,
			created_at: now,
			updated_at: now,
		};
		state.users.push(user.clone());
		Ok(user)
	}

	async fn find_by_id(&self, id: UserId) -> Result<Option<User>, IdentityError> {
		Ok(self.get(id))
	}

	async fn find_by_email(&self, email: &Email) -> Result<Option<User>, IdentityError> {
		Ok(self.state.lock().unwrap().users.iter().find(|user| &user.email == email).cloned())
	}

	async fn find_by_username(&self, username: &Username) -> Result<Option<User>, IdentityError> {
		let state = self.state.lock().unwrap();
		Ok(state.users.iter().find(|user| &user.username == username).cloned())
	}

	async fn list(&self) -> Result<Vec<User>, IdentityError> {
		Ok(self.state.lock().unwrap().users.clone())
	}

	async fn update(&self, user: &User) -> Result<(), IdentityError> {
		let mut state = self.state.lock().unwrap();
		let index = state
			.users
			.iter()
			.position(|existing| existing.id == user.id)
			.ok_or_else(|| IdentityError::PersistenceFailure("update of unknown user".into()))?;
		state.users[index] = user.clone();
		Ok(())
	}

	async fn save_email_change(
		&self,
		user: &User,
		previous_email: &Email,
	) -> Result<(), IdentityError> {
		let mut state = self.state.lock().unwrap();
		let index = state
			.users
			.iter()
			.position(|existing| existing.id == user.id)
			.ok_or_else(|| IdentityError::PersistenceFailure("update of unknown user".into()))?;
		state.users[index] = user.clone();
		state.email_history.push((user.id, previous_email.clone()));
		Ok(())
	}
}
