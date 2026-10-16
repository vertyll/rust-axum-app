//! The RFC 9457 problem document the back-end answers every refusal with. It
//! carries a message key and its arguments, never a sentence, so the reader's
//! text comes from the catalog in the reader's language.

use std::collections::BTreeMap;

use reqwest::StatusCode;
use serde::{Deserialize, Deserializer};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize)]
pub struct Problem {
    #[serde(deserialize_with = "status_code")]
    pub status: StatusCode,
    /// A key of the message catalog, such as `roles.errors.still_assigned`.
    pub code: String,
    /// The arguments of `code`; in a validation problem, keyed by field.
    #[serde(default)]
    pub args: Map<String, Value>,
    /// In a validation problem, the message keys of each invalid field.
    #[serde(default)]
    pub errors: BTreeMap<String, Vec<String>>,
}

impl Problem {
    #[must_use]
    pub fn new(status: StatusCode, code: &str) -> Self {
        Self {
            status,
            code: code.to_owned(),
            args: Map::new(),
            errors: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn field_args(&self, field: &str) -> Option<&Map<String, Value>> {
        self.args.get(field).and_then(Value::as_object)
    }
}

fn status_code<'de, D: Deserializer<'de>>(deserializer: D) -> Result<StatusCode, D::Error> {
    let code = u16::deserialize(deserializer)?;
    StatusCode::from_u16(code).map_err(serde::de::Error::custom)
}

#[cfg(test)]
mod tests {
    use super::Problem;

    #[test]
    fn reads_a_validation_problem() {
        let problem: Problem = serde_json::from_str(
            r#"{
                "type": "about:blank",
                "title": "Bad Request",
                "status": 400,
                "detail": "errors.validation",
                "code": "errors.validation",
                "args": {"description": {"max": 255}},
                "errors": {"description": ["roles.validators.description.length"]}
            }"#,
        )
        .unwrap();

        assert_eq!(problem.status.as_u16(), 400);
        assert_eq!(problem.errors["description"], ["roles.validators.description.length"]);
        assert_eq!(problem.field_args("description").unwrap()["max"], 255);
    }

    #[test]
    fn arguments_and_errors_are_optional() {
        let problem: Problem = serde_json::from_str(r#"{"status": 404, "code": "roles.errors.not_found"}"#).unwrap();

        assert!(problem.args.is_empty() && problem.errors.is_empty());
    }
}
