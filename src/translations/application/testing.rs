use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::translations::domain::{Translation, TranslationRepository, TranslationsError};

#[derive(Clone, Default)]
pub struct InMemoryTranslations {
	rows: Arc<Mutex<BTreeMap<String, Translation>>>,
}

impl InMemoryTranslations {
	fn rows(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Translation>> {
		self.rows.lock().expect("translations lock poisoned")
	}
}

impl TranslationRepository for InMemoryTranslations {
	async fn find_all(&self) -> Result<Vec<Translation>, TranslationsError> {
		Ok(self.rows().values().cloned().collect())
	}

	async fn find(&self, key: &str) -> Result<Option<Translation>, TranslationsError> {
		Ok(self.rows().get(key).cloned())
	}

	async fn create(&self, translation: &Translation) -> Result<(), TranslationsError> {
		self.rows().insert(translation.key.clone(), translation.clone());
		Ok(())
	}

	async fn update(&self, translation: &Translation) -> Result<(), TranslationsError> {
		self.rows().insert(translation.key.clone(), translation.clone());
		Ok(())
	}

	async fn delete(&self, key: &str) -> Result<(), TranslationsError> {
		self.rows().remove(key);
		Ok(())
	}
}
