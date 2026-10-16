use std::collections::BTreeMap;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{delete, get, put};
use axum::{Json, Router};
use serde::Deserialize;
use validator::Validate;

use super::error::ApiError;
use super::requests::TranslationRequest;
use super::responses::TranslationResponse;
use crate::identity::Permitted;
use crate::shared_http::pagination::{PageQuery, PageResponse};
use crate::shared_http::problem::{JsonBody, PathParam, QueryParams};
use crate::translations::application::TranslationsService;
use crate::translations::application::permissions::{TranslationsEdit, TranslationsView};
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
    Permitted(_caller, _): Permitted<TranslationsView>,
    QueryParams(query): QueryParams<ListQuery>,
) -> Result<Json<PageResponse<TranslationResponse>>, ApiError> {
    let page = PageQuery::new(query.page, query.size);
    page.validate()?;
    let matching = translations.list(query.q.as_deref().unwrap_or_default()).await?;
    Ok(Json(page.request().slice(matching).into()))
}

/// `?q=` keeps the messages whose key or text, in either language, holds it.
#[derive(Debug, Deserialize)]
struct ListQuery {
    q: Option<String>,
    page: Option<u32>,
    size: Option<u32>,
}

async fn update<R: TranslationRepository>(
    State(translations): Translations<R>,
    Permitted(_caller, _): Permitted<TranslationsEdit>,
    PathParam(key): PathParam<String>,
    JsonBody(request): JsonBody<TranslationRequest>,
) -> Result<Json<TranslationResponse>, ApiError> {
    request.validate()?;
    Ok(Json(translations.update(&key, request.into_text()).await?.into()))
}

async fn reset<R: TranslationRepository>(
    State(translations): Translations<R>,
    Permitted(_caller, _): Permitted<TranslationsEdit>,
    PathParam(key): PathParam<String>,
) -> Result<Json<TranslationResponse>, ApiError> {
    Ok(Json(translations.reset(&key).await?.into()))
}
