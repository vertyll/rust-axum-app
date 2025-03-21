use crate::auth::repositories::refresh_token_repository::{RefreshTokenRepository, RefreshTokenRepositoryTrait};
use crate::auth::services::auth_service::{AuthService, AuthServiceTrait};
use crate::auth::services::confirmation_token_service::{ConfirmationTokenService, ConfirmationTokenServiceTrait};
use crate::auth::services::refresh_token_service::{RefreshTokenService, RefreshTokenServiceTrait};
use crate::config::app_config::AppConfig;
use crate::emails::services::emails_service::{EmailsService, EmailsServiceTrait};
use crate::files::repositories::files_repository::FilesRepository;
use crate::files::services::files_service::{FilesService, FilesServiceTrait};
use crate::roles::repositories::roles_repository::{RolesRepository, RolesRepositoryTrait};
use crate::roles::repositories::user_roles_repository::{UserRolesRepository, UserRolesRepositoryTrait};
use crate::roles::services::roles_service::{RolesService, RolesServiceTrait};
use crate::roles::services::user_roles_service::{UserRolesService, UserRolesServiceTrait};
use crate::users::repositories::users_repository::{UsersRepository, UsersRepositoryTrait};
use crate::users::services::users_service::{UsersService, UsersServiceTrait};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub trait DatabaseConnectionTrait: Send + Sync {
	fn get_connection(&self) -> &DatabaseConnection;
}

pub struct DatabaseConnectionImpl {
	connection: Arc<DatabaseConnection>,
}

impl DatabaseConnectionImpl {
	pub fn new(connection: Arc<DatabaseConnection>) -> Self {
		Self { connection }
	}
}

impl DatabaseConnectionTrait for DatabaseConnectionImpl {
	fn get_connection(&self) -> &DatabaseConnection {
		&self.connection
	}
}

pub trait AppConfigTrait: Send + Sync {
	fn get_config(&self) -> &AppConfig;
}

pub struct AppConfigImpl {
	config: Arc<AppConfig>,
}

impl AppConfigImpl {
	pub fn new(config: Arc<AppConfig>) -> Self {
		Self { config }
	}
}

impl AppConfigTrait for AppConfigImpl {
	fn get_config(&self) -> &AppConfig {
		&self.config
	}
}

#[derive(Clone)]
pub struct AppModule {
	// Add db connection and config
	pub db_connection: Arc<dyn DatabaseConnectionTrait>,
	pub app_config: Arc<dyn AppConfigTrait>,

	// Add dependencies
	pub users_service: Arc<dyn UsersServiceTrait>,
	pub auth_service: Arc<dyn AuthServiceTrait>,
	pub refresh_token_service: Arc<dyn RefreshTokenServiceTrait>,
	pub email_service: Arc<dyn EmailsServiceTrait>,
	pub user_roles_service: Arc<dyn UserRolesServiceTrait>,
	pub confirmation_token_service: Arc<dyn ConfirmationTokenServiceTrait>,
	pub files_service: Arc<dyn FilesServiceTrait>,
	pub roles_service: Arc<dyn RolesServiceTrait>,
	pub refresh_token_repository: Arc<dyn RefreshTokenRepositoryTrait>,
	pub users_repository: Arc<dyn UsersRepositoryTrait>,
	pub roles_repository: Arc<dyn RolesRepositoryTrait>,
	pub user_roles_repository: Arc<dyn UserRolesRepositoryTrait>,
}

// Dependency injection
pub fn initialize_di(db: DatabaseConnection, config: Arc<AppConfig>) -> AppModule {
	// Add db connection and config
	let db_arc = Arc::new(db);
	let db_connection = Arc::new(DatabaseConnectionImpl::new(db_arc)) as Arc<dyn DatabaseConnectionTrait>;
	let app_config = Arc::new(AppConfigImpl::new(config)) as Arc<dyn AppConfigTrait>;

	// 1. Add repositories
	let refresh_token_repository = Arc::new(RefreshTokenRepository::new(db_connection.clone()));
	let users_repository = Arc::new(UsersRepository::new(db_connection.clone()));
	let roles_repository = Arc::new(RolesRepository::new(db_connection.clone()));
	let user_roles_repository = Arc::new(UserRolesRepository::new(db_connection.clone()));
	let files_repository = Arc::new(FilesRepository::new(db_connection.clone()));

	// 2. Add basic dependencies
	let email_service = Arc::new(EmailsService::new(app_config.clone()));
	let confirmation_token_service = Arc::new(ConfirmationTokenService::new(app_config.clone()));

	// 3. Add dependencies with sub-dependencies
	let roles_service = Arc::new(RolesService::new(roles_repository.clone()));
	let user_roles_service = Arc::new(UserRolesService::new(user_roles_repository.clone()));
	let users_service = Arc::new(UsersService::new(
		users_repository.clone(),
		user_roles_service.clone(),
		email_service.clone(),
		confirmation_token_service.clone(),
		app_config.clone(),
	));

	let auth_service = Arc::new(AuthService::new(
		users_service.clone(),
		user_roles_service.clone(),
		app_config.clone(),
	));

	let refresh_token_service = Arc::new(RefreshTokenService::new(
		refresh_token_repository.clone(),
		user_roles_service.clone(),
		app_config.clone(),
	));

	let files_service = Arc::new(FilesService::new(files_repository.clone(), app_config.clone()));

	AppModule {
		db_connection,
		app_config,
		users_service,
		auth_service,
		refresh_token_service,
		email_service,
		user_roles_service,
		confirmation_token_service,
		files_service,
		roles_service,
		refresh_token_repository,
		users_repository,
		roles_repository,
		user_roles_repository,
	}
}
