//! Error presentation of the translations HTTP adapter.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::shared_infrastructure::problem::{self, Problem};
use crate::translations::domain::TranslationsError;

#[derive(Debug)]
pub enum ApiError {
	Translations(TranslationsError),
	Validation(validator::ValidationErrors),
}

impl From<TranslationsError> for ApiError {
	fn from(err: TranslationsError) -> Self {
		Self::Translations(err)
	}
}

impl From<validator::ValidationErrors> for ApiError {
	fn from(errors: validator::ValidationErrors) -> Self {
		Self::Validation(errors)
	}
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		match self {
			ApiError::Validation(errors) => Problem::validation(&errors),
			ApiError::Translations(err) => translations_problem(err),
		}
		.into_response()
	}
}

fn translations_problem(err: TranslationsError) -> Problem {
	use TranslationsError as E;

	match err {
		E::NotFound => Problem::new(StatusCode::NOT_FOUND, "translations.errors.not_found"),
		E::UnknownLanguage => Problem::new(StatusCode::NOT_FOUND, "translations.errors.unknown_language"),
		E::InvalidMessage { language } => Problem::field(language.code(), "translations.errors.invalid_message"),
		E::UnknownPlaceholder { language, name } => Problem::field(
			language.code(),
			"translations.errors.unknown_placeholder",
		)
		.with_field_arg(language.code(), "name", name),
		E::PersistenceFailure(detail) => {
			tracing::error!("persistence failure: {detail}");
			Problem::new(StatusCode::INTERNAL_SERVER_ERROR, problem::INTERNAL)
		}
	}
}
