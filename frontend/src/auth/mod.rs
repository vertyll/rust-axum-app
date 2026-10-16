//! Who is signed in, and what their roles grant.
//!
//! Each page, route or component that needs a signed-in reader calls
//! [`require_auth`] or [`require_permission`] itself. What these grant only
//! decides what the pages offer: the back-end checks every call again, so a
//! hidden link is a convenience, not the protection.

pub mod api;

use reqwest::StatusCode;
use topcoat::context::{Cx, memoize};
use topcoat::router::error::{RouterErrorExt, internal_server_error};

use self::api::{Grants, User};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    UsersView,
    UsersManage,
    RolesView,
    RolesManage,
    FilesView,
    FilesManage,
    TranslationsView,
    TranslationsEdit,
}

impl Permission {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Permission::UsersView => "USERS_VIEW",
            Permission::UsersManage => "USERS_MANAGE",
            Permission::RolesView => "ROLES_VIEW",
            Permission::RolesManage => "ROLES_MANAGE",
            Permission::FilesView => "FILES_VIEW",
            Permission::FilesManage => "FILES_MANAGE",
            Permission::TranslationsView => "TRANSLATIONS_VIEW",
            Permission::TranslationsEdit => "TRANSLATIONS_EDIT",
        }
    }
}

#[derive(Debug)]
pub struct CurrentUser {
    pub user: User,
    pub grants: Grants,
}

impl CurrentUser {
    #[must_use]
    pub fn may(&self, permission: Permission) -> bool {
        self.grants.unrestricted || self.grants.permissions.contains(permission.name())
    }
}

#[memoize(as_ref)]
async fn load_current_user(cx: &Cx) -> Result<Option<CurrentUser>, String> {
    match tokio::try_join!(api::me(cx), api::my_grants(cx)) {
        Ok((user, grants)) => Ok(Some(CurrentUser { user, grants })),
        Err(err) if err.status() == Some(StatusCode::UNAUTHORIZED) => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the signed-in user could not be read: {0}")]
struct CurrentUserUnavailable(String);

/// The signed-in reader, or `None` for a guest; a failing back-end is an
/// error, not a guest.
pub async fn fetch_current_user(cx: &Cx) -> topcoat::Result<Option<&CurrentUser>> {
    match load_current_user(cx).await {
        Ok(current) => Ok(current.as_ref()),
        Err(reason) => Err(internal_server_error(CurrentUserUnavailable(reason.clone())).into()),
    }
}

pub async fn require_auth(cx: &Cx) -> topcoat::Result<&CurrentUser> {
    Ok(fetch_current_user(cx).await?.ok_or_unauthorized()?)
}

pub async fn require_permission(cx: &Cx, permission: Permission) -> topcoat::Result<&CurrentUser> {
    let current = require_auth(cx).await?;
    Ok(current.may(permission).then_some(current).ok_or_forbidden()?)
}
