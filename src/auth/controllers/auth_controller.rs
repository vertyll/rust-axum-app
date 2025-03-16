use crate::auth::dto::login_dto::LoginDto;
use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::services::auth_service::{AuthResponse, AuthService, AuthServiceTrait};
use crate::auth::services::refresh_token_service::{RefreshTokenService, RefreshTokenServiceTrait};
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::i18n::setup::translate;
use axum::response::IntoResponse;
use axum::{Json, Router, extract::State, routing::post};
use std::sync::Arc;
use time::{Duration as TimeDuration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

#[derive(Clone)]
struct AuthControllerStateDyn {
	auth_service: Arc<dyn AuthServiceTrait>,
	refresh_token_service: Arc<dyn RefreshTokenServiceTrait>,
	jwt_refresh_token_expires_in: i64,
}

pub fn routes(app_state: AppState) -> Router {
	let auth_service = Arc::new(AuthService::new(app_state.clone()));
	let refresh_token_service = Arc::new(RefreshTokenService::new(app_state.clone()));

	let dependencies_state = AuthControllerStateDyn {
		auth_service: auth_service.clone(),
		refresh_token_service: refresh_token_service.clone(),
		jwt_refresh_token_expires_in: app_state.config.security.jwt_refresh_token_expires_in,
	};

	Router::new()
		.route("/register", post(register))
		.route("/login", post(login))
		.route("/refresh-token", post(refresh_token))
		.route("/logout", post(logout))
		.route("/logout-all", post(logout_all_devices))
		.with_state(dependencies_state)
}

async fn register(
	State(dependencies): State<AuthControllerStateDyn>,
	cookies: Cookies,
	Json(dto): Json<RegisterDto>,
) -> Result<impl IntoResponse, AppError> {
	dto.validate()?;

	let (user, access_token, refresh_token) = dependencies
		.auth_service
		.register(dto, &dependencies.refresh_token_service)
		.await?;

	let cookie = create_refresh_token_cookie(refresh_token, dependencies.jwt_refresh_token_expires_in);
	cookies.add(cookie);

	let response = AuthResponse { user, access_token };

	Ok(Json(response))
}

async fn login(
	State(dependencies): State<AuthControllerStateDyn>,
	cookies: Cookies,
	Json(dto): Json<LoginDto>,
) -> Result<impl IntoResponse, AppError> {
	let (user, access_token, refresh_token) = dependencies
		.auth_service
		.login(dto, &dependencies.refresh_token_service)
		.await?;

	let cookie = create_refresh_token_cookie(refresh_token, dependencies.jwt_refresh_token_expires_in);
	cookies.add(cookie);

	let response = AuthResponse { user, access_token };

	Ok(Json(response))
}

async fn refresh_token(
	JwtAuth(claims): JwtAuth,
	State(dependencies): State<AuthControllerStateDyn>,
	cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
	let refresh_token = cookies
		.get("refresh_token")
		.ok_or_else(|| AppError::AuthenticationError(translate("auth.errors.missing_refresh_token")))?
		.value()
		.to_string();

	let response = dependencies
		.refresh_token_service
		.refresh_token(claims.sub, refresh_token)
		.await?;

	Ok(Json(response))
}

async fn logout(
	cookies: Cookies,
	JwtAuth(claims): JwtAuth,
	State(dependencies): State<AuthControllerStateDyn>,
) -> Result<impl IntoResponse, AppError> {
	let refresh_token = cookies.get("refresh_token").map(|cookie| cookie.value().to_string());

	if let Some(token) = refresh_token {
		dependencies
			.refresh_token_service
			.invalidate_refresh_token(claims.sub, token)
			.await?;
	}

	let mut cookie = Cookie::new("refresh_token", "");
	cookie.set_path("/");
	cookie.set_max_age(TimeDuration::seconds(0));
	cookie.set_http_only(true);
	cookie.set_secure(true);
	cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);

	cookies.add(cookie);

	Ok(())
}

async fn logout_all_devices(
	JwtAuth(claims): JwtAuth,
	State(dependencies): State<AuthControllerStateDyn>,
	cookies: Cookies,
) -> Result<impl IntoResponse, AppError> {
	dependencies
		.refresh_token_service
		.invalidate_all_user_tokens(claims.sub)
		.await?;

	let mut cookie = Cookie::new("refresh_token", "");
	cookie.set_path("/");
	cookie.set_max_age(TimeDuration::seconds(0));
	cookie.set_http_only(true);
	cookie.set_secure(true);
	cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);

	cookies.add(cookie);

	Ok(())
}

fn create_refresh_token_cookie(token: String, expires_in: i64) -> Cookie<'static> {
	let expiration = OffsetDateTime::now_utc() + TimeDuration::seconds(expires_in);

	let mut cookie = Cookie::new("refresh_token", token);
	cookie.set_path("/");
	cookie.set_secure(true);
	cookie.set_http_only(true);
	cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
	cookie.set_expires(expiration);

	cookie
}
