use jiff::Timestamp;

use super::error::TranslationsError;
use super::icu;
use super::language::Language;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedText {
	pub pl: String,
	pub en: String,
}

impl LocalizedText {
	pub fn get(&self, language: Language) -> &str {
		match language {
			Language::Pl => &self.pl,
			Language::En => &self.en,
		}
	}
}

/// One catalogue entry: the message clients render and the shipped default
/// it falls back to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Translation {
	pub key: String,
	pub message: LocalizedText,
	pub default: LocalizedText,
	pub customized: bool,
	pub updated_at: Timestamp,
}

impl Translation {
	pub fn new(key: String, default: LocalizedText, now: Timestamp) -> Self {
		Self {
			key,
			message: default.clone(),
			default,
			customized: false,
			updated_at: now,
		}
	}

	/// Replaces the message after checking every language parses and uses
	/// only the arguments its default uses.
	pub fn customize(&mut self, message: LocalizedText, now: Timestamp) -> Result<(), TranslationsError> {
		for language in Language::ALL {
			let used =
				icu::placeholders(message.get(language)).ok_or(TranslationsError::InvalidMessage { language })?;
			let known = icu::placeholders(self.default.get(language)).unwrap_or_default();
			if let Some(name) = used.difference(&known).next() {
				return Err(TranslationsError::UnknownPlaceholder {
					language,
					name: name.clone(),
				});
			}
		}
		self.message = message;
		self.customized = true;
		self.updated_at = now;
		Ok(())
	}

	pub fn reset(&mut self, now: Timestamp) {
		self.message = self.default.clone();
		self.customized = false;
		self.updated_at = now;
	}

	/// Adopts a new shipped default; an admin's override survives it.
	/// Returns whether anything changed.
	pub fn refresh_default(&mut self, default: &LocalizedText, now: Timestamp) -> bool {
		if &self.default == default {
			return false;
		}
		self.default = default.clone();
		if !self.customized {
			self.message = default.clone();
		}
		self.updated_at = now;
		true
	}
}

#[cfg(test)]
mod tests {
	use jiff::Timestamp;

	use super::{LocalizedText, Translation};
	use crate::translations::domain::{Language, TranslationsError};

	fn text(pl: &str, en: &str) -> LocalizedText {
		LocalizedText {
			pl: pl.to_string(),
			en: en.to_string(),
		}
	}

	fn password_rule() -> Translation {
		Translation::new(
			"users.validators.password.too_short".to_string(),
			text(
				"Hasło musi mieć co najmniej {min, plural, one {# znak} few {# znaki} many {# znaków} other {# znaku}}.",
				"Password must be at least {min, plural, one {# character} other {# characters}} long.",
			),
			Timestamp::UNIX_EPOCH,
		)
	}

	#[test]
	fn customize_accepts_a_message_using_known_arguments() {
		let mut translation = password_rule();

		translation
			.customize(text("Za krótkie (min. {min})", "Too short"), Timestamp::UNIX_EPOCH)
			.unwrap();

		assert!(translation.customized);
		assert_eq!(translation.message.pl, "Za krótkie (min. {min})");
	}

	#[test]
	fn customize_rejects_broken_syntax_and_unknown_arguments() {
		let mut translation = password_rule();

		assert_eq!(
			translation.customize(text("{min, plural,", "ok"), Timestamp::UNIX_EPOCH),
			Err(TranslationsError::InvalidMessage { language: Language::Pl })
		);
		assert_eq!(
			translation.customize(text("ok", "Hi {user}"), Timestamp::UNIX_EPOCH),
			Err(TranslationsError::UnknownPlaceholder {
				language: Language::En,
				name: "user".to_string()
			})
		);
		assert!(!translation.customized);
	}

	#[test]
	fn new_default_keeps_an_override_and_reset_restores_the_default() {
		let mut translation = password_rule();
		translation.customize(text("A", "B"), Timestamp::UNIX_EPOCH).unwrap();

		assert!(translation.refresh_default(&text("C", "D"), Timestamp::UNIX_EPOCH));
		assert_eq!(translation.message, text("A", "B"));

		translation.reset(Timestamp::UNIX_EPOCH);
		assert_eq!(translation.message, text("C", "D"));
		assert!(!translation.customized);
	}
}
