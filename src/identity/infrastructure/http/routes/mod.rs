mod auth;
mod users;

pub use auth::{auth_protected_router, auth_public_router};
pub use users::users_router;

use std::sync::Arc;

use axum::extract::State;

use crate::identity::application::service::IdentityService;

pub(crate) type Identity<P> = State<Arc<IdentityService<P>>>;
