//! The choices the reader shares with Keycloak's pages, in cookies both read
//! and write. The theme and the language read them here; the `keycloak`
//! module, which reads the theme and the language, sits above them, so the
//! two never depend on each other.

use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies};
use topcoat::router::header::HOST;
use topcoat::router::request::headers;

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
    use super::{Preference, shared_cookie_domain};

    /// Keycloak's theme names the cookies after the realm, so a renamed realm
    /// must rename them here too.
    #[test]
    fn the_cookies_are_named_after_the_realm() {
        let export: serde_json::Value =
            serde_json::from_str(include_str!("../../keycloak/realm-export.json")).expect("the realm export is JSON");
        let realm = export["realm"].as_str().expect("the realm export names the realm");
        assert_eq!(Preference::Theme.cookie(), format!("ui_theme_{realm}"));
        assert_eq!(Preference::Language.cookie(), format!("ui_locale_{realm}"));
    }

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
