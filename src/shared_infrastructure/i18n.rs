//! Request-scoped internationalization: the locale lives in a Tokio
//! task-local scoped around each request (a global `RwLock` set per
//! request, as before, races between concurrent requests).
//! task-local scoped around each request (a global `RwLock` set per
//! request, as before, races between concurrent requests).
//!
//! The previous implementation stored the current locale in a global
//! `RwLock<String>` and called the global `rust_i18n::set_locale` per
//! request — a race condition under concurrent requests (one request's
//! `Accept-Language` could leak into another's response). Here the locale
//! lives in a Tokio task-local scoped around the request future, so every
//! request translates with its own locale.

use axum::{extract::Request, middleware::Next, response::Response};

pub const DEFAULT_LOCALE: &str = "en";
pub const SUPPORTED_LOCALES: [&str; 2] = ["en", "pl"];

tokio::task_local! {
	static LOCALE: String;
}

/// Axum middleware: resolves the locale from `Accept-Language` and scopes it
/// around the rest of the request pipeline.
pub async fn middleware(request: Request, next: Next) -> Response {
	let locale = request
		.headers()
		.get("accept-language")
		.and_then(|value| value.to_str().ok())
		.map(parse_accept_language)
		.unwrap_or_else(|| DEFAULT_LOCALE.to_string());

	LOCALE.scope(locale, next.run(request)).await
}

/// The locale of the current request, or the default outside a request scope
/// (e.g. in background jobs).
pub fn locale() -> String {
	LOCALE
		.try_with(Clone::clone)
		.unwrap_or_else(|_| DEFAULT_LOCALE.to_string())
}

/// Translates `key` in the locale of the current request.
pub fn translate(key: &str) -> String {
	rust_i18n::t!(key, locale = &locale()).to_string()
}

fn parse_accept_language(header: &str) -> String {
	let requested = header
		.split(',')
		.next()
		.and_then(|lang| lang.split(';').next())
		.and_then(|lang| lang.split('-').next())
		.unwrap_or(DEFAULT_LOCALE)
		.to_lowercase();

	if SUPPORTED_LOCALES.contains(&requested.as_str()) {
		requested
	} else {
		DEFAULT_LOCALE.to_string()
	}
}
#[cfg(test)]
mod tests {
	use super::{DEFAULT_LOCALE, locale, parse_accept_language};

	#[test]
	fn picks_first_supported_language() {
		assert_eq!(parse_accept_language("pl,en;q=0.9"), "pl");
		assert_eq!(parse_accept_language("PL-pl"), "pl");
	}

	#[test]
	fn falls_back_to_default() {
		assert_eq!(parse_accept_language("de-DE,de;q=0.9"), DEFAULT_LOCALE);
		assert_eq!(locale(), DEFAULT_LOCALE);
	}
}
