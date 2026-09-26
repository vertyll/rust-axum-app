//! Repository port of the translations context (Toasty adapter in
//! `infrastructure::persistence`).

use super::error::TranslationsError;
use super::translation::Translation;

pub trait TranslationRepository: Clone + Send + Sync + 'static {
	fn find_all(&self) -> impl Future<Output = Result<Vec<Translation>, TranslationsError>> + Send;

	fn find(&self, key: &str) -> impl Future<Output = Result<Option<Translation>, TranslationsError>> + Send;

	fn create(&self, translation: &Translation) -> impl Future<Output = Result<(), TranslationsError>> + Send;

	fn update(&self, translation: &Translation) -> impl Future<Output = Result<(), TranslationsError>> + Send;

	fn delete(&self, key: &str) -> impl Future<Output = Result<(), TranslationsError>> + Send;
}
