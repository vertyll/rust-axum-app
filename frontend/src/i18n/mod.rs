//! What the reader reads, in their language. Every text is a key of the back
//! end's message catalog, formatted as ICU `MessageFormat`, so an
//! administrator's edit of a message reaches these pages too.

pub mod api;

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::str::FromStr;

use formatjs_icu_messageformat::{DateTimeValue, IcuMessageFormat, Value, Values};
use jiff::Timestamp;
use jiff::tz::TimeZone;
use reqwest::header::ACCEPT_LANGUAGE;
use serde_json::Map;
use topcoat::context::{Cx, memoize};
use topcoat::cookie::{Cookies, cookies};
use topcoat::router::error::internal_server_error;
use topcoat::router::request::headers;

use crate::backend::Problem;

pub const LANG_COOKIE: &str = "lang";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Lang {
    #[default]
    Pl,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Pl, Lang::En];

    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Lang::Pl => "pl",
            Lang::En => "en",
        }
    }

    fn preferred(accept_language: &str) -> Option<Lang> {
        accept_language
            .split(',')
            .filter_map(|part| part.split(';').next())
            .filter_map(|tag| tag.trim().split('-').next())
            .find_map(|language| language.to_ascii_lowercase().parse().ok())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("unsupported language: {0}")]
pub struct UnsupportedLang(String);

impl FromStr for Lang {
    type Err = UnsupportedLang;

    fn from_str(code: &str) -> Result<Self, Self::Err> {
        Lang::ALL
            .into_iter()
            .find(|lang| lang.code() == code)
            .ok_or_else(|| UnsupportedLang(code.to_owned()))
    }
}

impl fmt::Display for Lang {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

/// The reader's language: the one they chose, else the first their browser
/// asks for that the catalog has, else Polish.
pub fn lang(cx: &Cx) -> Lang {
    cookies(cx)
        .get(LANG_COOKIE)
        .and_then(|cookie| cookie.value().parse().ok())
        .or_else(|| {
            headers(cx)
                .get(ACCEPT_LANGUAGE)
                .and_then(|value| value.to_str().ok())
                .and_then(Lang::preferred)
        })
        .unwrap_or_default()
}

#[derive(Debug)]
pub struct Texts {
    lang: Lang,
    messages: BTreeMap<String, String>,
    fallback: Option<BTreeMap<String, String>>,
}

pub type Args = Vec<(&'static str, Value)>;

impl Texts {
    #[must_use]
    pub fn lang(&self) -> Lang {
        self.lang
    }

    #[must_use]
    pub fn t(&self, key: &str) -> String {
        self.format(key, &Values::new())
    }

    #[must_use]
    pub fn t_with(&self, key: &str, args: Args) -> String {
        let values = args.into_iter().map(|(name, value)| (name.to_owned(), value)).collect();
        self.format(key, &values)
    }

    #[must_use]
    pub fn problem(&self, problem: &Problem) -> String {
        self.format(&problem.code, &values_of(&problem.args))
    }

    #[must_use]
    pub fn field_errors(&self, problem: &Problem, field: &str) -> Vec<String> {
        let values = problem.field_args(field).map(values_of).unwrap_or_default();
        problem
            .errors
            .get(field)
            .into_iter()
            .flatten()
            .map(|key| self.format(key, &values))
            .collect()
    }

    /// A moment, as a date and a time in UTC.
    #[must_use]
    pub fn datetime(&self, at: Timestamp) -> String {
        let at = at.to_zoned(TimeZone::UTC);
        let value = DateTimeValue::try_new(
            i32::from(at.year()),
            u8::try_from(at.month()).expect("a month is 1 to 12"),
            u8::try_from(at.day()).expect("a day is 1 to 31"),
            u8::try_from(at.hour()).expect("an hour is 0 to 23"),
            u8::try_from(at.minute()).expect("a minute is 0 to 59"),
            u8::try_from(at.second()).expect("a second is 0 to 59"),
            0,
        )
        .expect("a timestamp's fields are a valid date and time");
        self.t_with("common.datetime", vec![("at", Value::from(value))])
    }

    /// A key missing from the reader's catalog is reported and taken from the
    /// default language's; the tests check that every key these pages use is
    /// in both.
    fn format(&self, key: &str, values: &Values) -> String {
        let (message, lang) = if let Some(message) = self.messages.get(key) {
            (message, self.lang)
        } else {
            tracing::warn!(key, lang = %self.lang, "message key missing from the catalog");
            let Some(message) = self.fallback.as_ref().and_then(|fallback| fallback.get(key)) else {
                return key.to_owned();
            };
            (message, Lang::default())
        };
        IcuMessageFormat::try_new(message)
            .and_then(|format| format.format_to_string(lang.code(), values))
            .unwrap_or_else(|err| {
                tracing::warn!(key, lang = %self.lang, "message could not be formatted: {err}");
                message.clone()
            })
    }
}

fn values_of(args: &Map<String, serde_json::Value>) -> Values {
    args.iter()
        .filter_map(|(name, value)| {
            let value = match value {
                serde_json::Value::String(text) => Value::from(text.as_str()),
                serde_json::Value::Bool(flag) => Value::from(*flag),
                serde_json::Value::Number(number) => number
                    .as_i64()
                    .map(Value::from)
                    .or_else(|| number.as_f64().map(Value::from))?,
                _ => return None,
            };
            Some((name.clone(), value))
        })
        .collect::<HashMap<_, _>>()
}

#[memoize(as_ref)]
async fn load_texts(cx: &Cx, lang: Lang) -> Option<Texts> {
    let fallback_lang = (lang != Lang::default()).then(Lang::default);
    let (messages, fallback) = tokio::join!(api::catalog(cx, lang), async {
        match fallback_lang {
            Some(fallback_lang) => api::catalog(cx, fallback_lang).await.map(Some),
            None => Ok(None),
        }
    });
    match messages.and_then(|messages| fallback.map(|fallback| (messages, fallback))) {
        Ok((messages, fallback)) => Some(Texts {
            lang,
            messages,
            fallback,
        }),
        Err(err) => {
            tracing::error!(%lang, "the message catalog could not be read: {err}");
            None
        }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("the message catalog is unavailable")]
struct CatalogUnavailable;

pub async fn texts(cx: &Cx) -> topcoat::Result<&Texts> {
    load_texts(cx, lang(cx))
        .await
        .ok_or_else(|| internal_server_error(CatalogUnavailable).into())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use formatjs_icu_messageformat::Value;
    use jiff::Timestamp;

    use super::{Lang, Texts};
    use crate::backend::Problem;

    fn texts(lang: Lang, messages: &[(&str, &str)]) -> Texts {
        Texts {
            lang,
            messages: messages
                .iter()
                .map(|(key, message)| ((*key).to_owned(), (*message).to_owned()))
                .collect::<BTreeMap<_, _>>(),
            fallback: None,
        }
    }

    #[test]
    fn picks_the_first_supported_language() {
        assert_eq!(Lang::preferred("de-DE,en-GB;q=0.9,pl;q=0.8"), Some(Lang::En));
        assert_eq!(Lang::preferred("fr"), None);
    }

    #[test]
    fn formats_polish_plurals_by_their_cldr_categories() {
        let texts = texts(
            Lang::Pl,
            &[(
                "files.count",
                "{count, plural, one {# plik} few {# pliki} many {# plików} other {# pliku}}",
            )],
        );

        let count = |n: i64| texts.t_with("files.count", vec![("count", Value::from(n))]);

        assert_eq!(count(1), "1 plik");
        assert_eq!(count(3), "3 pliki");
        assert_eq!(count(5), "5 plików");
        assert_eq!(count(22), "22 pliki");
    }

    #[test]
    fn renders_a_problem_with_its_arguments() {
        let texts = texts(
            Lang::En,
            &[("auth.errors.permission_denied", "You lack the permission {permission}.")],
        );
        let problem: Problem = serde_json::from_str(
            r#"{"status": 403, "code": "auth.errors.permission_denied", "args": {"permission": "ROLES_MANAGE"}}"#,
        )
        .unwrap();

        assert_eq!(texts.problem(&problem), "You lack the permission ROLES_MANAGE.");
    }

    #[test]
    fn renders_the_errors_of_one_field() {
        let texts = texts(
            Lang::En,
            &[("roles.validators.description.length", "At most {max} characters.")],
        );
        let problem: Problem = serde_json::from_str(
            r#"{"status": 400, "code": "errors.validation",
                "args": {"description": {"max": 255}},
                "errors": {"description": ["roles.validators.description.length"]}}"#,
        )
        .unwrap();

        assert_eq!(texts.field_errors(&problem, "description"), ["At most 255 characters."]);
        assert_eq!(texts.field_errors(&problem, "name"), Vec::<String>::new());
    }

    #[test]
    fn formats_a_moment_in_utc() {
        let texts = texts(
            Lang::En,
            &[("common.datetime", "{at, date, medium}, {at, time, short} UTC")],
        );
        let at: Timestamp = "2026-10-16T08:05:00Z".parse().unwrap();

        let text = texts.datetime(at);

        assert_eq!(text, "Oct 16, 2026, 8:05\u{202f}AM UTC");
    }
}
