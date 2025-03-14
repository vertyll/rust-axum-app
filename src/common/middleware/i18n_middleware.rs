use crate::i18n::setup::set_language;
use axum::{extract::Request, middleware::Next, response::Response};

pub async fn i18n_middleware(request: Request, next: Next) -> Response {
	if let Some(lang) = request.headers().get("accept-language") {
		if let Ok(lang_str) = lang.to_str() {
			let lang_code = parse_accept_language(lang_str);
			set_language(&lang_code);
		}
	}

	next.run(request).await
}

fn parse_accept_language(header: &str) -> String {
	header
		.split(',')
		.next()
		.and_then(|lang| lang.split(';').next())
		.and_then(|lang| lang.split('-').next())
		.unwrap_or("en")
		.to_lowercase()
}
