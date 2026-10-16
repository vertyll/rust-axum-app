//! The default catalogue: the messages every module ships, merged. Both
//! languages of each must list exactly the same keys, and no key may be
//! shipped twice.

use std::collections::BTreeMap;

use anyhow::{Context as _, bail};

use crate::shared_kernel::messages::Messages;
use crate::translations::domain::{LocalizedText, icu};

/// # Errors
///
/// When a file is not a JSON object of valid ICU messages, a key is missing
/// from one language, or two owners ship the same key.
pub fn shipped(all: &[Messages]) -> anyhow::Result<BTreeMap<String, LocalizedText>> {
    let mut catalogue = BTreeMap::new();
    let mut owners: BTreeMap<String, &str> = BTreeMap::new();
    for messages in all {
        for (key, text) in merge(messages)? {
            if let Some(first) = owners.insert(key.clone(), messages.owner) {
                bail!("{key} is shipped by both {first} and {}", messages.owner);
            }
            catalogue.insert(key, text);
        }
    }
    Ok(catalogue)
}

fn merge(messages: &Messages) -> anyhow::Result<BTreeMap<String, LocalizedText>> {
    let owner = messages.owner;
    let pl: BTreeMap<String, String> =
        serde_json::from_str(messages.pl).with_context(|| format!("{owner}: pl.json"))?;
    let mut en: BTreeMap<String, String> =
        serde_json::from_str(messages.en).with_context(|| format!("{owner}: en.json"))?;

    let mut merged = BTreeMap::new();
    for (key, pl_message) in pl {
        let Some(en_message) = en.remove(&key) else {
            bail!("{owner}: en.json lacks {key}");
        };
        for message in [&pl_message, &en_message] {
            if icu::placeholders(message).is_none() {
                bail!("{owner}: {key} is not valid ICU MessageFormat: {message}");
            }
        }
        merged.insert(
            key,
            LocalizedText {
                pl: pl_message,
                en: en_message,
            },
        );
    }
    if let Some(key) = en.keys().next() {
        bail!("{owner}: pl.json lacks {key}");
    }
    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::shipped;
    use crate::shared_kernel::messages::Messages;

    const GREETING: Messages = Messages {
        owner: "greeting",
        pl: r#"{"greeting.hello": "Cześć {name}"}"#,
        en: r#"{"greeting.hello": "Hello {name}"}"#,
    };

    #[test]
    fn the_messages_of_every_owner_are_merged() {
        let other = Messages {
            owner: "other",
            pl: r#"{"other.bye": "Pa"}"#,
            en: r#"{"other.bye": "Bye"}"#,
        };

        let catalogue = shipped(&[GREETING, other]).unwrap();

        assert_eq!(catalogue.keys().collect::<Vec<_>>(), ["greeting.hello", "other.bye"]);
    }

    #[test]
    fn a_key_shipped_twice_is_refused() {
        let error = shipped(&[GREETING, GREETING]).unwrap_err();

        assert!(error.to_string().contains("shipped by both"), "{error}");
    }

    #[test]
    fn a_key_missing_from_one_language_is_refused() {
        let partial = Messages { en: "{}", ..GREETING };

        assert!(shipped(&[partial]).is_err());
    }
}
