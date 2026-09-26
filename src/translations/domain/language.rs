use std::fmt;
use std::str::FromStr;

use super::error::TranslationsError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
	Pl,
	En,
}

impl Language {
	pub const ALL: [Language; 2] = [Language::Pl, Language::En];

	pub fn code(self) -> &'static str {
		match self {
			Language::Pl => "pl",
			Language::En => "en",
		}
	}
}

impl FromStr for Language {
	type Err = TranslationsError;

	fn from_str(code: &str) -> Result<Self, Self::Err> {
		Language::ALL
			.into_iter()
			.find(|language| language.code() == code)
			.ok_or(TranslationsError::UnknownLanguage)
	}
}

impl fmt::Display for Language {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.write_str(self.code())
	}
}

#[cfg(test)]
mod tests {
	use super::Language;
	use crate::translations::domain::TranslationsError;

	#[test]
	fn parses_exact_codes_only() {
		assert_eq!("pl".parse::<Language>(), Ok(Language::Pl));
		assert_eq!("en".parse::<Language>(), Ok(Language::En));
		assert_eq!("PL".parse::<Language>(), Err(TranslationsError::UnknownLanguage));
	}
}
