//! The way into Keycloak's pages and the person's account there: the theme and
//! the language the reader shares with Keycloak's pages, the account actions
//! that open one of those pages, and the language kept on the account. The
//! rest of the front-end uses only what this module makes public; the cookies
//! and their domain are its own.

use reqwest::Method;
use serde::Serialize;
use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies};
use topcoat::router::header::HOST;
use topcoat::router::href;
use topcoat::router::request::headers;

use crate::auth::fetch_current_user;
use crate::backend;
use crate::backend::proxy::{Rest, api};
use crate::config::AppConfig;

/// A choice the reader shares with Keycloak's pages, kept in a cookie both
/// read and write: a shared record rather than a message, so the two agree
/// however the reader moves between them, through a sign-in, a cancelled
/// account action, the browser's history or another tab. Keycloak's scripts
/// read it, so it is not `HttpOnly`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Preference {
    Theme,
    Language,
}

impl Preference {
    const fn cookie(self) -> &'static str {
        match self {
            Preference::Theme => "ui_theme_rust-axum-app",
            Preference::Language => "ui_locale_rust-axum-app",
        }
    }
}

/// The value last chosen here or on Keycloak's pages, as its code, when one was.
pub fn chosen(cx: &Cx, preference: Preference) -> Option<String> {
    cookies(cx)
        .get(preference.cookie())
        .map(|cookie| cookie.value().to_owned())
}

/// Records a choice made here for Keycloak's pages, under the parent of this
/// request's `Host`.
pub fn share(cx: &Cx, preference: Preference, code: &str) {
    let cookie = Cookie::build((preference.cookie(), code.to_owned()))
        .path("/")
        .same_site(SameSite::Lax)
        .secure(app_context::<AppConfig>(cx).secure_cookies())
        .permanent();
    let host = headers(cx).get(HOST).and_then(|host| host.to_str().ok());
    let cookie = match host.and_then(shared_cookie_domain) {
        Some(domain) => cookie.domain(domain.to_owned()),
        None => cookie,
    };
    cookies(cx).add(cookie.build());
}

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

/// Where the browser goes for an account action: the sign-in, asking Keycloak
/// for that one page.
pub fn account_action(cx: &Cx, action: &AccountAction) -> String {
    #[derive(Serialize)]
    struct Action {
        kc_action: String,
    }
    let kc_action = match action {
        AccountAction::UpdatePassword => "UPDATE_PASSWORD".to_owned(),
        AccountAction::ConfigureTotp => "CONFIGURE_TOTP".to_owned(),
        AccountAction::DeleteCredential(id) => format!("delete_credential:{id}"),
    };
    href!(api, Rest(["auth", "authorize"]))
        .query(Action { kc_action })
        .resolve(cx)
}

/// The domain a cookie shared with Keycloak lives under: the parent of this
/// host. `localhost` and an IP address have none, as every port there already
/// shares cookies.
fn shared_cookie_domain(host: &str) -> Option<&str> {
    let name = host.split(':').next().unwrap_or(host);
    let is_address = name
        .split('.')
        .all(|label| label.bytes().all(|byte| byte.is_ascii_digit()));
    if name.split('.').count() > 2 && !is_address {
        name.split_once('.').map(|(_, parent)| parent)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::shared_cookie_domain;

    #[test]
    fn a_shared_cookie_lives_under_the_parent_domain() {
        assert_eq!(shared_cookie_domain("app.vertyll.dev"), Some("vertyll.dev"));
        assert_eq!(shared_cookie_domain("app.vertyll.dev:443"), Some("vertyll.dev"));
    }

    #[test]
    fn localhost_and_an_address_share_cookies_without_a_domain() {
        assert_eq!(shared_cookie_domain("localhost:3000"), None);
        assert_eq!(shared_cookie_domain("127.0.0.1:3000"), None);
    }
}
