use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::{IdentityError, UserId, UserRepository};

impl<P: IdentityPorts> IdentityService<P> {
	pub async fn deactivate_user(&self, id: UserId) -> Result<(), IdentityError> {
		let mut user = self.get_user(id).await?;
		user.deactivate();
		self.users.update(&user).await
	}
}
#[cfg(test)]
mod tests {
	use crate::identity::application::testing::{harness, register_cmd};

	#[tokio::test]
	async fn flips_is_active_off() {
		let h = harness();
		let user = h.service.create_user(register_cmd("alice", "a@example.com")).await.unwrap();
		h.service.deactivate_user(user.id).await.unwrap();
		assert!(!h.users.get(user.id).unwrap().is_active);
	}
}
