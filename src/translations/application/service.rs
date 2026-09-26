//! Use cases of the translations context.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use jiff::Timestamp;

use crate::translations::domain::{Language, LocalizedText, Translation, TranslationRepository, TranslationsError};

#[derive(Clone)]
pub struct TranslationsService<R> {
	repository: R,
	defaults: Arc<BTreeMap<String, LocalizedText>>,
}

impl<R: TranslationRepository> TranslationsService<R> {
	pub fn new(repository: R, defaults: BTreeMap<String, LocalizedText>) -> Self {
		Self {
			repository,
			defaults: Arc::new(defaults),
		}
	}

	/// Every message in one language, keyed by message key.
	pub async fn catalogue(&self, language: Language) -> Result<BTreeMap<String, String>, TranslationsError> {
		let translations = self.repository.find_all().await?;
		Ok(translations
			.into_iter()
			.map(|translation| (translation.key, translation.message.get(language).to_string()))
			.collect())
	}

	pub async fn list(&self) -> Result<Vec<Translation>, TranslationsError> {
		let mut translations = self.repository.find_all().await?;
		translations.sort_by(|left, right| left.key.cmp(&right.key));
		Ok(translations)
	}

	pub async fn update(&self, key: &str, message: LocalizedText) -> Result<Translation, TranslationsError> {
		let mut translation = self.find(key).await?;
		translation.customize(message, Timestamp::now())?;
		self.repository.update(&translation).await?;
		Ok(translation)
	}

	pub async fn reset(&self, key: &str) -> Result<Translation, TranslationsError> {
		let mut translation = self.find(key).await?;
		translation.reset(Timestamp::now());
		self.repository.update(&translation).await?;
		Ok(translation)
	}

	/// Brings the stored catalogue in line with the shipped defaults: adds
	/// new keys, adopts changed defaults and drops keys the code no longer uses.
	pub async fn synchronize(&self) -> Result<(), TranslationsError> {
		let now = Timestamp::now();
		let stored = self.repository.find_all().await?;
		let stored_keys: BTreeSet<&str> = stored.iter().map(|translation| translation.key.as_str()).collect();

		for mut translation in stored.iter().cloned() {
			match self.defaults.get(&translation.key) {
				Some(default) => {
					if translation.refresh_default(default, now) {
						self.repository.update(&translation).await?;
					}
				}
				None => self.repository.delete(&translation.key).await?,
			}
		}

		for (key, default) in self.defaults.iter() {
			if !stored_keys.contains(key.as_str()) {
				self.repository
					.create(&Translation::new(key.clone(), default.clone(), now))
					.await?;
			}
		}
		Ok(())
	}

	async fn find(&self, key: &str) -> Result<Translation, TranslationsError> {
		self.repository.find(key).await?.ok_or(TranslationsError::NotFound)
	}
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeMap;

	use super::TranslationsService;
	use crate::translations::application::testing::InMemoryTranslations;
	use crate::translations::domain::{Language, LocalizedText, TranslationsError};

	fn text(pl: &str, en: &str) -> LocalizedText {
		LocalizedText {
			pl: pl.to_string(),
			en: en.to_string(),
		}
	}

	fn service(
		defaults: &[(&str, LocalizedText)],
		repository: InMemoryTranslations,
	) -> TranslationsService<InMemoryTranslations> {
		let defaults: BTreeMap<String, LocalizedText> = defaults
			.iter()
			.map(|(key, text)| ((*key).to_string(), text.clone()))
			.collect();
		TranslationsService::new(repository, defaults)
	}

	#[tokio::test]
	async fn synchronize_adds_updates_and_removes_keys() {
		let repository = InMemoryTranslations::default();
		service(&[("a", text("A", "A")), ("old", text("O", "O"))], repository.clone())
			.synchronize()
			.await
			.unwrap();

		let current = service(&[("a", text("A2", "A2")), ("b", text("B", "B"))], repository.clone());
		current.synchronize().await.unwrap();

		let catalogue = current.catalogue(Language::Pl).await.unwrap();
		assert_eq!(catalogue.keys().collect::<Vec<_>>(), ["a", "b"]);
		assert_eq!(catalogue["a"], "A2");
	}

	#[tokio::test]
	async fn override_survives_synchronization_until_reset() {
		let repository = InMemoryTranslations::default();
		let translations = service(&[("greeting", text("Cześć {name}", "Hi {name}"))], repository);
		translations.synchronize().await.unwrap();

		translations
			.update("greeting", text("Witaj {name}", "Hello {name}"))
			.await
			.unwrap();
		translations.synchronize().await.unwrap();
		assert_eq!(
			translations.catalogue(Language::Pl).await.unwrap()["greeting"],
			"Witaj {name}"
		);

		translations.reset("greeting").await.unwrap();
		assert_eq!(
			translations.catalogue(Language::En).await.unwrap()["greeting"],
			"Hi {name}"
		);
	}

	#[tokio::test]
	async fn unknown_key_is_not_found() {
		let translations = service(&[], InMemoryTranslations::default());

		assert_eq!(
			translations.update("missing", text("a", "b")).await,
			Err(TranslationsError::NotFound)
		);
	}
}
