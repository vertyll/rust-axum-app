use crate::auth::dto::change_email_dto::ChangeEmailDto;
use crate::auth::dto::change_password_dto::ChangePasswordDto;
use crate::auth::dto::forgot_password_dto::ForgotPasswordDto;
use crate::auth::dto::login_dto::LoginDto;
use crate::auth::dto::register_dto::RegisterDto;
use crate::auth::dto::reset_password_dto::ResetPasswordDto;
use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::auth::services::auth_service::{AuthResponse, AuthService, AuthServiceTrait};
use crate::auth::services::refresh_token_service::{RefreshTokenService, RefreshTokenServiceTrait};
use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::i18n::setup::translate;
use crate::users::services::users_service::{UsersService, UsersServiceTrait};
use axum::response::IntoResponse;
use axum::{
	Json, Router,
	extract::Query,
	extract::State,
	routing::{get, post},
};
use serde::Deserialize;
use std::sync::Arc;
use time::{Duration as TimeDuration, OffsetDateTime};
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

#[derive(Clone)]
struct AuthControllerStateDyn {
	auth_service: Arc<dyn AuthServiceTrait>,
	refresh_token_service: Arc<dyn RefreshTokenServiceTrait>,
	users_service: Arc<dyn UsersServiceTrait>,
	jwt_refresh_token_expires_in: i64,
}

#[derive(Deserialize)]
struct EmailConfirmationQuery {
	token: String,
}

#[derive(Deserialize)]
struct EmailChangeConfirmationQuery {
	token: String,
}

pub fn routes(app_state: AppState) -> Router {
	let auth_service = Arc::new(AuthService::new(app_state.clone()));
	let refresh_token_service = Arc::new(RefreshTokenService::new(app_state.clone()));
	let users_service = Arc::new(UsersService::new(app_state.clone()));

	let dependencies = AuthControllerStateDyn {
		auth_service,
		refresh_token_service: refresh_token_service.clone(),
		users_service,
		jwt_refresh_token_expires_in: app_state.config.security.tokens.jwt_refresh_token.expires_in,
	};

	Router::new()
		.route("/register", post(register))
		.route("/login", post(login))
		.route("/refresh-token", post(refresh_token))
		.route("/logout", post(logout))
		.route("/logout-all", post(logout_all_devices))
		.route("/confirm-email", get(confirm_email))
		.route("/password/change", post(change_password))
		.route("/password/reset", post(request_reset_password))
		.route("/confirm-password-reset", post(confirm_reset_password))
		.route("/email/change", post(request_email_change))
		.route("/confirm-email-change", get(confirm_email_change))
		.with_state(dependencies)
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
	dto.validate()?;

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

async fn confirm_email(
	State(dependencies): State<AuthControllerStateDyn>,
	Query(query): Query<EmailConfirmationQuery>,
) -> Result<impl IntoResponse, AppError> {
	dependencies.users_service.confirm_email(&query.token).await?;
	Ok(())
}

async fn request_reset_password(
	State(dependencies): State<AuthControllerStateDyn>,
	Json(dto): Json<ForgotPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
	dto.validate()?;
	dependencies.users_service.request_reset_password(dto).await?;
	Ok(())
}

async fn confirm_reset_password(
	State(dependencies): State<AuthControllerStateDyn>,
	Json(dto): Json<ResetPasswordDto>,
) -> Result<impl IntoResponse, AppError> {
	dto.validate()?;
	dependencies.users_service.reset_password(dto).await?;
	Ok(())
}

async fn change_password(
	JwtAuth(claims): JwtAuth,
	State(dependencies): State<AuthControllerStateDyn>,
	Json(dto): Json<ChangePasswordDto>,
) -> Result<impl IntoResponse, AppError> {
	dto.validate()?;
	dependencies.users_service.change_password(claims.sub, dto).await?;
	Ok(())
}

async fn request_email_change(
	JwtAuth(claims): JwtAuth,
	State(dependencies): State<AuthControllerStateDyn>,
	Json(dto): Json<ChangeEmailDto>,
) -> Result<impl IntoResponse, AppError> {
	dto.validate()?;
	dependencies.users_service.request_email_change(claims.sub, dto).await?;
	Ok(())
}

async fn confirm_email_change(
	State(dependencies): State<AuthControllerStateDyn>,
	Query(query): Query<EmailChangeConfirmationQuery>,
) -> Result<impl IntoResponse, AppError> {
	dependencies.users_service.confirm_email_change(&query.token).await?;
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
