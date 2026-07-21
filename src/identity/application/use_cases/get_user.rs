use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, User, UserId, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn get_user(&self, id: UserId) -> Result<User, IdentityError> {
		self.users.find_by_id(id).await?.ok_or(IdentityError::UserNotFound)
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{harness, register_cmd};
	use crate::identity::domain::{IdentityError, UserId};

	#[tokio::test]
	async fn finds_by_id_or_reports_missing() {
		let h = harness();
		let user = h.service.create_user(register_cmd("alice", "a@example.com")).await.unwrap();
		assert_eq!(h.service.get_user(user.id).await.unwrap().id, user.id);

		let missing = h.service.get_user(UserId(999)).await.unwrap_err();
		assert!(matches!(missing, IdentityError::UserNotFound));
	}
}
