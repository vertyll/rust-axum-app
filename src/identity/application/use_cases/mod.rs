//! One file per use case. Each file is a single `impl` block adding one
//! public method to [`IdentityService`](super::IdentityService).

mod authenticate;
mod change_password;
mod clean_expired_sessions;
mod confirm_email;
mod confirm_email_change;
mod create_user;
mod deactivate_user;
mod get_user;
mod list_users;
mod login;
mod logout;
mod logout_all_devices;
mod refresh_access_token;
mod register;
mod request_email_change;
mod request_password_reset;
mod reset_password;
mod update_user;
