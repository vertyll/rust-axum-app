//! Outbound response DTOs, one file per struct. A response model cannot
//! leak what it does not contain — unlike serializing the ORM entity.

mod access_token_response;
mod auth_response;
mod user_response;

pub use access_token_response::AccessTokenResponse;
pub use auth_response::AuthResponse;
pub use user_response::UserResponse;
