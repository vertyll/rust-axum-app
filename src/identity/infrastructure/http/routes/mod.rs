mod auth;
mod users;

use std::sync::Arc;

use axum::extract::State;

use crate::identity::application::service::IdentityService;
use crate::identity::infrastructure::keycloak::KeycloakSettings;

pub use auth::auth_router;
pub use users::users_router;

pub(crate) type Identity<P> = State<Arc<IdentityService<P>>>;

/// What the sign-in routes need besides the service: where Keycloak is and
/// where the browser goes once signed in.
#[derive(Debug, Clone)]
pub struct SignInSettings {
	pub keycloak: KeycloakSettings,
	pub post_login_url: String,
}
