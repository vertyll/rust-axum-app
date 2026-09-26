use jiff::Timestamp;

use crate::translations::domain::{LocalizedText, Translation};

#[derive(Debug, toasty::Model)]
#[table = "translations"]
pub struct TranslationRecord {
	#[key]
	#[auto]
	pub id: i64,
	#[unique]
	pub message_key: String,
	pub message_pl: String,
	pub message_en: String,
	pub default_pl: String,
	pub default_en: String,
	pub customized: bool,
	#[auto]
	pub created_at: Timestamp,
	#[auto]
	pub updated_at: Timestamp,
}

impl TranslationRecord {
	pub fn to_domain(&self) -> Translation {
		Translation {
			key: self.message_key.clone(),
			message: LocalizedText {
				pl: self.message_pl.clone(),
				en: self.message_en.clone(),
			},
			default: LocalizedText {
				pl: self.default_pl.clone(),
				en: self.default_en.clone(),
			},
			customized: self.customized,
			updated_at: self.updated_at,
		}
	}
}
