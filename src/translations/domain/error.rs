use thiserror::Error;

use super::language::Language;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TranslationsError {
	#[error("translation not found")]
	NotFound,
	#[error("unknown language")]
	UnknownLanguage,
	#[error("{language} message is not valid ICU MessageFormat")]
	InvalidMessage { language: Language },
	#[error("{language} message uses unknown placeholder {name}")]
	UnknownPlaceholder { language: Language, name: String },
	#[error("persistence failure: {0}")]
	PersistenceFailure(String),
}
