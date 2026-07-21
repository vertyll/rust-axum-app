use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use tower_cookies::cookie::SameSite;
use tower_cookies::cookie::time::Duration;
use tower_cookies::{Cookie, Cookies};
use validator::Validate;

use super::Identity;
use crate::identity::application::ports::IdentityPorts;
use crate::identity::application::service::IdentityService;
use crate::identity::domain::IdentityError;
use crate::identity::infrastructure::http::error::ApiError;
use crate::identity::infrastructure::http::extract::Auth;
use crate::identity::infrastructure::http::requests::{
	ChangeEmailRequest, ChangePasswordRequest, ForgotPasswordRequest, LoginRequest, RegisterRequest,
	ResetPasswordRequest, TokenQuery,
};
use crate::identity::infrastructure::http::responses::{AccessTokenResponse, AuthResponse};

const REFRESH_COOKIE: &str = "refresh_token";

pub fn auth_public_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
	Router::new()
		.route("/register", post(register))
		.route("/login", post(login))
		.route("/refresh-token", post(refresh_token))
		.route("/confirm-email", get(confirm_email))
		.route("/password/reset", post(request_password_reset))
		.route("/confirm-password-reset", post(confirm_password_reset))
		.route("/confirm-email-change", get(confirm_email_change))
		.with_state(identity)
}

pub fn auth_protected_router<P: IdentityPorts>(identity: Arc<IdentityService<P>>) -> Router {
	Router::new()
		.route("/logout", post(logout))
		.route("/logout-all", post(logout_all_devices))
		.route("/password/change", post(change_password))
		.route("/email/change", post(request_email_change))
		.with_state(identity)
}

async fn register<P: IdentityPorts>(
	State(identity): Identity<P>,
	cookies: Cookies,
	Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
	request.validate()?;
	let (user, tokens) = identity.register(request.into_command()?).await?;

	cookies.add(refresh_cookie(tokens.refresh_token, identity.refresh_ttl_seconds()));
	Ok(Json(AuthResponse {
		user: user.into(),
		access_token: tokens.access_token,
	}))
}

async fn login<P: IdentityPorts>(
	State(identity): Identity<P>,
	cookies: Cookies,
	Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
	request.validate()?;
	let (user, tokens) = identity.login(request.into()).await?;

	cookies.add(refresh_cookie(tokens.refresh_token, identity.refresh_ttl_seconds()));
	Ok(Json(AuthResponse {
		user: user.into(),
		access_token: tokens.access_token,
	}))
}

async fn refresh_token<P: IdentityPorts>(
	State(identity): Identity<P>,
	cookies: Cookies,
) -> Result<Json<AccessTokenResponse>, ApiError> {
	let refresh_token = cookies
		.get(REFRESH_COOKIE)
		.map(|cookie| cookie.value().to_string())
		.ok_or(IdentityError::RefreshTokenMissing)?;

	let access_token = identity.refresh_access_token(&refresh_token).await?;
	Ok(Json(AccessTokenResponse { access_token }))
}

async fn logout<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(claims): Auth,
	cookies: Cookies,
) -> Result<(), ApiError> {
	if let Some(cookie) = cookies.get(REFRESH_COOKIE) {
		identity.logout(claims.user_id(), cookie.value()).await?;
	}
	cookies.add(expired_refresh_cookie());
	Ok(())
}

async fn logout_all_devices<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(claims): Auth,
	cookies: Cookies,
) -> Result<(), ApiError> {
	identity.logout_all_devices(claims.user_id()).await?;
	cookies.add(expired_refresh_cookie());
	Ok(())
}

async fn confirm_email<P: IdentityPorts>(
	State(identity): Identity<P>,
	Query(query): Query<TokenQuery>,
) -> Result<(), ApiError> {
	identity.confirm_email(&query.token).await?;
	Ok(())
}

async fn request_password_reset<P: IdentityPorts>(
	State(identity): Identity<P>,
	Json(request): Json<ForgotPasswordRequest>,
) -> Result<(), ApiError> {
	request.validate()?;
	identity.request_password_reset(request.into_command()?).await?;
	Ok(())
}

async fn confirm_password_reset<P: IdentityPorts>(
	State(identity): Identity<P>,
	Json(request): Json<ResetPasswordRequest>,
) -> Result<(), ApiError> {
	request.validate()?;
	identity.reset_password(request.into()).await?;
	Ok(())
}

async fn change_password<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(claims): Auth,
	Json(request): Json<ChangePasswordRequest>,
) -> Result<(), ApiError> {
	request.validate()?;
	identity.change_password(claims.user_id(), request.into()).await?;
	Ok(())
}

async fn request_email_change<P: IdentityPorts>(
	State(identity): Identity<P>,
	Auth(claims): Auth,
	Json(request): Json<ChangeEmailRequest>,
) -> Result<(), ApiError> {
	request.validate()?;
	identity
		.request_email_change(claims.user_id(), request.into_command()?)
		.await?;
	Ok(())
}

async fn confirm_email_change<P: IdentityPorts>(
	State(identity): Identity<P>,
	Query(query): Query<TokenQuery>,
) -> Result<(), ApiError> {
	identity.confirm_email_change(&query.token).await?;
	Ok(())
}

fn refresh_cookie(token: String, max_age_seconds: i64) -> Cookie<'static> {
	Cookie::build((REFRESH_COOKIE, token))
		.path("/")
		.http_only(true)
		.secure(true)
		.same_site(SameSite::Strict)
		.max_age(Duration::seconds(max_age_seconds))
		.build()
}

fn expired_refresh_cookie() -> Cookie<'static> {
	Cookie::build((REFRESH_COOKIE, ""))
		.path("/")
		.http_only(true)
		.secure(true)
		.same_site(SameSite::Strict)
		.max_age(Duration::seconds(0))
		.build()
}
