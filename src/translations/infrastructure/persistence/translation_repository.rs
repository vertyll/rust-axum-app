use super::TranslationRecord;
use crate::translations::domain::{Translation, TranslationRepository, TranslationsError};

#[derive(Clone)]
pub struct ToastyTranslationRepository {
	db: toasty::Db,
}

impl ToastyTranslationRepository {
	pub fn new(db: toasty::Db) -> Self {
		Self { db }
	}
}

impl TranslationRepository for ToastyTranslationRepository {
	async fn find_all(&self) -> Result<Vec<Translation>, TranslationsError> {
		let mut db = self.db.clone();

		let records = TranslationRecord::all().exec(&mut db).await.map_err(persistence)?;

		Ok(records.iter().map(TranslationRecord::to_domain).collect())
	}

	async fn find(&self, key: &str) -> Result<Option<Translation>, TranslationsError> {
		let mut db = self.db.clone();

		let record = TranslationRecord::filter_by_message_key(key)
			.first()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(record.as_ref().map(TranslationRecord::to_domain))
	}

	async fn create(&self, translation: &Translation) -> Result<(), TranslationsError> {
		let mut db = self.db.clone();

		toasty::create!(TranslationRecord {
			message_key: translation.key.clone(),
			message_pl: translation.message.pl.clone(),
			message_en: translation.message.en.clone(),
			default_pl: translation.default.pl.clone(),
			default_en: translation.default.en.clone(),
			customized: translation.customized,
		})
		.exec(&mut db)
		.await
		.map_err(persistence)?;

		Ok(())
	}

	async fn update(&self, translation: &Translation) -> Result<(), TranslationsError> {
		let mut db = self.db.clone();

		toasty::update!(TranslationRecord::filter_by_message_key(translation.key.clone()) {
			message_pl: translation.message.pl.clone(),
			message_en: translation.message.en.clone(),
			default_pl: translation.default.pl.clone(),
			default_en: translation.default.en.clone(),
			customized: translation.customized,
		})
		.exec(&mut db)
		.await
		.map_err(persistence)?;

		Ok(())
	}

	async fn delete(&self, key: &str) -> Result<(), TranslationsError> {
		let mut db = self.db.clone();

		TranslationRecord::filter_by_message_key(key)
			.delete()
			.exec(&mut db)
			.await
			.map_err(persistence)?;

		Ok(())
	}
}

fn persistence(err: toasty::Error) -> TranslationsError {
	TranslationsError::PersistenceFailure(err.to_string())
}
