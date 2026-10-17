//! The signed-in reader's account on Keycloak: the actions that open one of
//! its pages, the second factor, and the language kept on the account.

use reqwest::Method;
use serde::{Deserialize, Serialize};
use topcoat::context::Cx;

use crate::auth::fetch_current_user;
use crate::backend::{self, BackendError};

/// Keycloak shows a signed-in person their account's language, so a signed-in
/// reader's choice goes there too; when Keycloak cannot store it, the choice
/// still holds here.
pub async fn keep_account_language(cx: &Cx, code: &str) -> topcoat::Result<()> {
    #[derive(Serialize)]
    struct AccountLanguage<'a> {
        language: &'a str,
    }
    if fetch_current_user(cx).await?.is_none() {
        return Ok(());
    }
    let request = backend::request(cx, Method::PUT, "/api/users/me/language").json(&AccountLanguage { language: code });
    if let Err(err) = backend::send(cx, request).await {
        tracing::warn!("could not store the account's language: {err:?}");
    }
    Ok(())
}

/// A page of Keycloak's that a signed-in person is sent to, which returns here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AccountAction {
    UpdatePassword,
    ConfigureTotp,
    /// Removes the credential with this id, after Keycloak asks the reader to confirm.
    DeleteCredential(String),
}

/// The reader's second factor, as the back-end reads it from Keycloak.
#[derive(Debug, Deserialize)]
pub struct SecondFactor {
    credential_id: Option<String>,
}

impl SecondFactor {
    pub const fn is_on(&self) -> bool {
        self.credential_id.is_some()
    }

    /// The action turning it off when it is on, which Keycloak asks to
    /// confirm, and setting it up when it is not.
    pub fn toggle(&self) -> AccountAction {
        match &self.credential_id {
            Some(id) => AccountAction::DeleteCredential(id.clone()),
            None => AccountAction::ConfigureTotp,
        }
    }
}

/// The signed-in reader's second factor.
pub async fn second_factor(cx: &Cx) -> Result<SecondFactor, BackendError> {
    backend::json(cx, backend::request(cx, Method::GET, "/api/users/me/second-factor")).await
}
