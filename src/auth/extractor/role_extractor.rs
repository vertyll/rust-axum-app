use crate::auth::extractor::jwt_auth_extractor::JwtAuth;
use crate::common::enums::role_enum::RoleEnum;
use crate::common::error::app_error::AppError;
use crate::i18n::setup::translate;
use axum::{extract::FromRequestParts, http::request::Parts};
use std::future::Future;

pub struct RoleGuard(pub Vec<RoleEnum>);
impl<S> FromRequestParts<S> for RoleGuard
where
	S: Send + Sync,
{
	type Rejection = AppError;
	fn from_request_parts(parts: &mut Parts, state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			let JwtAuth(claims) = JwtAuth::from_request_parts(parts, state).await?;
			Ok(RoleGuard(claims.roles))
		}
	}
}

pub struct AdminRole;
impl<S> FromRequestParts<S> for AdminRole
where
	S: Send + Sync,
{
	type Rejection = AppError;
	fn from_request_parts(parts: &mut Parts, state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			let RoleGuard(roles) = RoleGuard::from_request_parts(parts, state).await?;

			if !roles.contains(&RoleEnum::Admin) {
				return Err(AppError::AuthorizationError(translate(
					"auth.errors.admin_role_required",
				)));
			}

			Ok(AdminRole)
		}
	}
}

pub struct ManagerRole;
impl<S> FromRequestParts<S> for ManagerRole
where
	S: Send + Sync,
{
	type Rejection = AppError;
	fn from_request_parts(parts: &mut Parts, state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			let RoleGuard(roles) = RoleGuard::from_request_parts(parts, state).await?;

			if !roles.contains(&RoleEnum::Manager) {
				return Err(AppError::AuthorizationError(translate(
					"auth.errors.manager_role_required",
				)));
			}

			Ok(ManagerRole)
		}
	}
}

pub struct UserRole;
impl<S> FromRequestParts<S> for UserRole
where
	S: Send + Sync,
{
	type Rejection = AppError;
	fn from_request_parts(parts: &mut Parts, state: &S) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
		async move {
			let RoleGuard(roles) = RoleGuard::from_request_parts(parts, state).await?;

			if !roles.contains(&RoleEnum::User) {
				return Err(AppError::AuthorizationError(translate(
					"auth.errors.user_role_required",
				)));
			}

			Ok(UserRole)
		}
	}
}
