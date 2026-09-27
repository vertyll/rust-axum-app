//! RFC 9457 problem documents — the one error body every module answers with.
//!
//! The server never renders prose: `code` names a key of the translation
//! catalogue (`GET /api/translations/{language}`), `args` carries the ICU
//! arguments for it and `errors` maps each rejected field to its own keys.
//! In a validation problem `args` is keyed by field too, since two fields may
//! be rejected by the same rule with different limits.
//! `detail` repeats `code`, matching the other services of this workspace.

use std::collections::BTreeMap;

use axum::extract::FromRequest;
use axum::extract::FromRequestParts;
use axum::extract::rejection::{JsonRejection, PathRejection, QueryRejection};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use serde_json::{Map, Value};
use validator::ValidationErrors;

pub const PROBLEM_JSON: &str = "application/problem+json";

pub const VALIDATION_FAILED: &str = "errors.validation";
pub const BAD_REQUEST: &str = "errors.bad_request";
pub const NOT_FOUND: &str = "errors.not_found";
pub const INTERNAL: &str = "errors.internal";
const INVALID_VALUE: &str = "validation.invalid";

#[derive(Debug, Serialize)]
pub struct Problem {
	#[serde(rename = "type")]
	problem_type: &'static str,
	title: &'static str,
	#[serde(serialize_with = "status_code")]
	status: StatusCode,
	detail: String,
	code: String,
	#[serde(skip_serializing_if = "Map::is_empty")]
	args: Map<String, Value>,
	#[serde(skip_serializing_if = "BTreeMap::is_empty")]
	errors: BTreeMap<String, Vec<String>>,
}

impl Problem {
	pub fn new(status: StatusCode, code: impl Into<String>) -> Self {
		let code = code.into();
		Self {
			problem_type: "about:blank",
			title: status.canonical_reason().unwrap_or_default(),
			status,
			detail: code.clone(),
			code,
			args: Map::new(),
			errors: BTreeMap::new(),
		}
	}

	/// A `400` naming one rejected field — for rules only the domain can check.
	pub fn field(field: &str, key: &str) -> Self {
		let mut problem = Self::new(StatusCode::BAD_REQUEST, VALIDATION_FAILED);
		problem.errors.insert(field.to_string(), vec![key.to_string()]);
		problem
	}

	#[must_use]
	pub fn with_field_arg(mut self, field: &str, name: &str, value: impl Into<Value>) -> Self {
		self.field_args(field).insert(name.to_string(), value.into());
		self
	}

	/// A `400` listing every rejected field. A validator's message is the
	/// catalogue key; its parameters (never the submitted `value`) become the
	/// field's ICU arguments.
	pub fn validation(errors: &ValidationErrors) -> Self {
		let mut problem = Self::new(StatusCode::BAD_REQUEST, VALIDATION_FAILED);
		for (field, field_errors) in errors.field_errors() {
			let keys = field_errors
				.iter()
				.map(|error| error.message.as_deref().unwrap_or(INVALID_VALUE).to_string())
				.collect();
			problem.errors.insert(field.to_string(), keys);
			let params: Vec<(String, Value)> = field_errors
				.iter()
				.flat_map(|error| &error.params)
				.filter(|(name, _)| *name != "value")
				.map(|(name, value)| (name.to_string(), value.clone()))
				.collect();
			if !params.is_empty() {
				problem.field_args(&field).extend(params);
			}
		}
		problem
	}

	fn field_args(&mut self, field: &str) -> &mut Map<String, Value> {
		let entry = self
			.args
			.entry(field.to_string())
			.or_insert_with(|| Value::Object(Map::new()));
		if !entry.is_object() {
			*entry = Value::Object(Map::new());
		}
		entry.as_object_mut().expect("field arguments are an object")
	}
}

impl IntoResponse for Problem {
	fn into_response(self) -> Response {
		let body = serde_json::to_vec(&self).expect("a problem document is plain data and always serializes");
		(self.status, [(header::CONTENT_TYPE, PROBLEM_JSON)], body).into_response()
	}
}

fn status_code<S: serde::Serializer>(status: &StatusCode, serializer: S) -> Result<S::Ok, S::Error> {
	serializer.serialize_u16(status.as_u16())
}

impl From<JsonRejection> for Problem {
	fn from(rejection: JsonRejection) -> Self {
		tracing::debug!("rejected request body: {rejection}");
		Self::new(rejection.status(), BAD_REQUEST)
	}
}

impl From<PathRejection> for Problem {
	fn from(rejection: PathRejection) -> Self {
		tracing::debug!("rejected path: {rejection}");
		Self::new(StatusCode::BAD_REQUEST, BAD_REQUEST)
	}
}

impl From<QueryRejection> for Problem {
	fn from(rejection: QueryRejection) -> Self {
		tracing::debug!("rejected query: {rejection}");
		Self::new(StatusCode::BAD_REQUEST, BAD_REQUEST)
	}
}

/// `axum::Json` whose rejection is a problem document instead of plain text.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(Problem))]
pub struct JsonBody<T>(pub T);

/// `axum::extract::Path` whose rejection is a problem document.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(Problem))]
pub struct PathParam<T>(pub T);

/// `axum::extract::Query` whose rejection is a problem document.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(Problem))]
pub struct QueryParams<T>(pub T);

/// Router fallback: an unknown path answers in the same shape as everything else.
pub async fn not_found() -> Problem {
	Problem::new(StatusCode::NOT_FOUND, NOT_FOUND)
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use axum::http::StatusCode;
	use serde_json::json;
	use validator::{ValidationError, ValidationErrors};

	use super::Problem;

	#[test]
	fn validation_lists_keys_and_never_echoes_the_submitted_value() {
		let mut error = ValidationError::new("length");
		error.message = Some(Cow::Borrowed("users.validators.password.too_short"));
		error.add_param(Cow::Borrowed("min"), &8);
		error.add_param(Cow::Borrowed("value"), &"hunter2");
		let mut errors = ValidationErrors::new();
		errors.add("password", error);

		let body = serde_json::to_value(Problem::validation(&errors)).unwrap();

		assert_eq!(body["status"], 400);
		assert_eq!(body["code"], "errors.validation");
		assert_eq!(
			body["errors"],
			json!({ "password": ["users.validators.password.too_short"] })
		);
		assert_eq!(body["args"], json!({ "password": { "min": 8 } }));
		assert!(!body.to_string().contains("hunter2"));
	}

	#[test]
	fn plain_problem_omits_empty_members() {
		let body = serde_json::to_value(Problem::new(StatusCode::NOT_FOUND, "errors.not_found")).unwrap();

		assert_eq!(
			body,
			json!({
				"type": "about:blank",
				"title": "Not Found",
				"status": 404,
				"detail": "errors.not_found",
				"code": "errors.not_found",
			})
		);
	}
}
