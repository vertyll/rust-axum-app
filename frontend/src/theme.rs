//! The color theme the reader chose: light, dark, or the one their system uses.

use std::fmt;
use std::str::FromStr;

use topcoat::context::{Cx, app_context};
use topcoat::cookie::{Cookie, Cookies, SameSite, cookies};

use crate::config::AppConfig;

pub const THEME_COOKIE: &str = "theme";

/// The cookie Keycloak's pages leave a theme chosen there in, named after the realm, as the sign-in returns to an
/// address that carries nothing else.
pub const SIGN_IN_THEME_COOKIE: &str = "ui_theme_rust-axum-app";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub const ALL: [Theme; 3] = [Theme::System, Theme::Light, Theme::Dark];

    /// The value of the cookie and of `<html data-theme>`, which the stylesheet reads.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Theme::System => "system",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unsupported theme: {0}")]
pub struct UnsupportedTheme(String);

impl FromStr for Theme {
    type Err = UnsupportedTheme;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        Theme::ALL
            .into_iter()
            .find(|theme| theme.code() == code)
            .ok_or_else(|| UnsupportedTheme(code.to_owned()))
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// The reader's theme: the one they chose, else the system's.
pub fn theme(cx: &Cx) -> Theme {
    cookies(cx)
        .get(THEME_COOKIE)
        .and_then(|cookie| cookie.value().parse().ok())
        .unwrap_or_default()
}

/// Remembers the reader's theme on this site.
pub fn remember(cx: &Cx, theme: Theme) {
    let cookie = Cookie::build((THEME_COOKIE, theme.code()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .secure(app_context::<AppConfig>(cx).secure_cookies())
        .permanent()
        .build();
    cookies(cx).add(cookie);
}

/// Takes over a theme chosen on Keycloak's pages, once: remembers it here and removes the cookie it came in, from the
/// domain Keycloak set it under. `host` is this request's `Host`.
pub fn take_sign_in_theme(cx: &Cx, host: Option<&str>) {
    let jar = cookies(cx);
    let Some(handed_over) = jar.get(SIGN_IN_THEME_COOKIE) else {
        return;
    };
    if let Ok(theme) = handed_over.value().parse() {
        remember(cx, theme);
    }
    let removal = Cookie::build(SIGN_IN_THEME_COOKIE).path("/");
    let removal = match host.and_then(shared_cookie_domain) {
        Some(domain) => removal.domain(domain.to_owned()),
        None => removal,
    };
    jar.remove(removal.build());
}

/// The domain a cookie shared with Keycloak lives under: the parent of this host. `localhost` and an IP address have
/// none, as every port there already shares cookies.
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
    use super::{Theme, shared_cookie_domain};

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

    #[test]
    fn a_theme_reads_back_from_its_code() {
        for theme in Theme::ALL {
            assert_eq!(theme.code().parse::<Theme>().ok(), Some(theme));
        }
        assert!("sepia".parse::<Theme>().is_err());
    }
}
