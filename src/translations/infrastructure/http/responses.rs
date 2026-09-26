use jiff::Timestamp;
use serde::Serialize;

use crate::translations::domain::{LocalizedText, Translation};

#[derive(Debug, Serialize)]
pub struct LocalizedTextResponse {
	pub pl: String,
	pub en: String,
}

impl From<LocalizedText> for LocalizedTextResponse {
	fn from(text: LocalizedText) -> Self {
		Self {
			pl: text.pl,
			en: text.en,
		}
	}
}

#[derive(Debug, Serialize)]
pub struct TranslationResponse {
	pub key: String,
	pub message: LocalizedTextResponse,
	pub default: LocalizedTextResponse,
	pub customized: bool,
	pub updated_at: Timestamp,
}

impl From<Translation> for TranslationResponse {
	fn from(translation: Translation) -> Self {
		Self {
			key: translation.key,
			message: translation.message.into(),
			default: translation.default.into(),
			customized: translation.customized,
			updated_at: translation.updated_at,
		}
	}
}
