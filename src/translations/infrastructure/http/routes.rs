use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use validator::Validate;

use super::error::ApiError;
use super::requests::TranslationRequest;
use super::responses::TranslationResponse;
use crate::identity::RequireAdmin;
use crate::shared_infrastructure::problem::{JsonBody, PathParam};
use crate::translations::application::TranslationsService;
use crate::translations::domain::{Language, TranslationRepository};

type Translations<R> = State<Arc<TranslationsService<R>>>;

/// `GET /{language}` — the whole catalogue of one language, open to anyone.
pub fn translations_public_router<R: TranslationRepository>(translations: Arc<TranslationsService<R>>) -> Router {
	Router::new()
		.route("/{language}", get(catalogue))
		.with_state(translations)
}

/// Catalogue management; the composition root puts it behind the auth layer.
pub fn translations_admin_router<R: TranslationRepository>(translations: Arc<TranslationsService<R>>) -> Router {
	Router::new()
		.route("/", get(list))
		.route("/{key}", put(update))
		.route("/{key}/customization", delete(reset))
		.with_state(translations)
}

async fn catalogue<R: TranslationRepository>(
	State(translations): Translations<R>,
	PathParam(language): PathParam<String>,
) -> Result<Json<BTreeMap<String, String>>, ApiError> {
	let language: Language = language.parse()?;
	Ok(Json(translations.catalogue(language).await?))
}

async fn list<R: TranslationRepository>(
	State(translations): Translations<R>,
	RequireAdmin(_claims): RequireAdmin,
) -> Result<Json<Vec<TranslationResponse>>, ApiError> {
	let list = translations.list().await?;
	Ok(Json(list.into_iter().map(TranslationResponse::from).collect()))
}

async fn update<R: TranslationRepository>(
	State(translations): Translations<R>,
	RequireAdmin(_claims): RequireAdmin,
	PathParam(key): PathParam<String>,
	JsonBody(request): JsonBody<TranslationRequest>,
) -> Result<Json<TranslationResponse>, ApiError> {
	request.validate()?;
	Ok(Json(translations.update(&key, request.into_text()).await?.into()))
}

async fn reset<R: TranslationRepository>(
	State(translations): Translations<R>,
	RequireAdmin(_claims): RequireAdmin,
	PathParam(key): PathParam<String>,
) -> Result<Json<TranslationResponse>, ApiError> {
	Ok(Json(translations.reset(&key).await?.into()))
}
