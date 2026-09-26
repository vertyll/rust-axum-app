use serde::Deserialize;
use validator::Validate;

use crate::translations::domain::LocalizedText;

#[derive(Debug, Deserialize, Validate)]
pub struct TranslationRequest {
	#[validate(length(min = 1, max = 2000, message = "translations.validators.message.length"))]
	pub pl: String,
	#[validate(length(min = 1, max = 2000, message = "translations.validators.message.length"))]
	pub en: String,
}

impl TranslationRequest {
	pub fn into_text(self) -> LocalizedText {
		LocalizedText {
			pl: self.pl,
			en: self.en,
		}
	}
}

#[cfg(test)]
mod tests {
	use validator::Validate;

	use super::TranslationRequest;

	#[test]
	fn blank_message_is_rejected() {
		let request = TranslationRequest {
			pl: String::new(),
			en: "ok".to_string(),
		};

		assert!(request.validate().is_err());
	}
}
