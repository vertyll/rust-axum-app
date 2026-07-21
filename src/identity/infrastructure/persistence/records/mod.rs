//! Toasty records of the identity context, one per table. Records are a
//! persistence detail — the flat, nullable-column shape the database wants;
//! the domain shape is rebuilt in `to_domain`. No ORM relations point at
//! other bounded contexts: cross-module references are ids only.

mod email_history_record;
mod refresh_token_record;
mod role_record;
mod user_record;
mod user_role_record;

pub use email_history_record::EmailHistoryRecord;
pub use refresh_token_record::RefreshTokenRecord;
pub use role_record::RoleRecord;
pub use user_record::UserRecord;
pub use user_role_record::UserRoleRecord;
