use once_cell::sync::Lazy;
use std::sync::RwLock;

// Store the current language
static CURRENT_LANGUAGE: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("en".to_string()));

pub fn set_language(lang: &str) {
	// Only set supported languages
	let supported_languages = ["en", "pl"];
	let lang = if supported_languages.contains(&lang) {
		lang
	} else {
		"en" // Default to English if unsupported
	};

	if let Ok(mut current) = CURRENT_LANGUAGE.write() {
		*current = lang.to_string();
		rust_i18n::set_locale(lang);
	}
}

pub fn get_language() -> String {
	CURRENT_LANGUAGE.read().unwrap().clone()
}

pub fn translate(key: &str) -> String {
	// Check if the language has been set before translating
	let lang = get_language();
	rust_i18n::set_locale(&lang);
	rust_i18n::t!(key).to_string()
}
