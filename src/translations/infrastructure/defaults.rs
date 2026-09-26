//! The catalogue shipped with the binary (`translations/{pl,en}.json`), the
//! source of every default. Both languages must list exactly the same keys.

use std::collections::BTreeMap;

use anyhow::{Context as _, bail};

use crate::translations::domain::{LocalizedText, icu};

const PL: &str = include_str!("../../../translations/pl.json");
const EN: &str = include_str!("../../../translations/en.json");

pub fn shipped() -> anyhow::Result<BTreeMap<String, LocalizedText>> {
	let pl: BTreeMap<String, String> = serde_json::from_str(PL).context("translations/pl.json")?;
	let mut en: BTreeMap<String, String> = serde_json::from_str(EN).context("translations/en.json")?;

	let mut catalogue = BTreeMap::new();
	for (key, pl_message) in pl {
		let Some(en_message) = en.remove(&key) else {
			bail!("translations/en.json lacks {key}");
		};
		for message in [&pl_message, &en_message] {
			if icu::placeholders(message).is_none() {
				bail!("{key}: not valid ICU MessageFormat: {message}");
			}
		}
		catalogue.insert(
			key,
			LocalizedText {
				pl: pl_message,
				en: en_message,
			},
		);
	}
	if let Some(key) = en.keys().next() {
		bail!("translations/pl.json lacks {key}");
	}
	Ok(catalogue)
}

#[cfg(test)]
mod tests {
	use super::shipped;

	#[test]
	fn shipped_catalogue_is_complete_and_valid() {
		let catalogue = shipped().unwrap();

		assert!(catalogue.contains_key("errors.validation"));
	}
}
