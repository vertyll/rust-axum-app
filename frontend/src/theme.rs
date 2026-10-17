//! The color theme the reader chose: light, dark, or the one their system uses.

use std::fmt;
use std::str::FromStr;

use topcoat::context::Cx;
use topcoat::cookie::{Cookies, cookies};

pub const THEME_COOKIE: &str = "theme";

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

#[cfg(test)]
mod tests {
    use super::Theme;

    #[test]
    fn a_theme_reads_back_from_its_code() {
        for theme in Theme::ALL {
            assert_eq!(theme.code().parse::<Theme>().ok(), Some(theme));
        }
        assert!("sepia".parse::<Theme>().is_err());
    }
}
