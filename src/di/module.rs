use crate::auth::repositories::refresh_token_repository::RefreshTokenRepositoryImpl;
use crate::auth::services::auth_service::AuthServiceImpl;
use crate::auth::services::confirmation_token_service::ConfirmationTokenServiceImpl;
use crate::auth::services::refresh_token_service::RefreshTokenServiceImpl;
use crate::config::app_config::AppConfig;
use crate::emails::services::emails_service::EmailsServiceImpl;
use crate::emails::strategies::emails_strategy::SmtpEmailStrategyImpl;
use crate::files::repositories::files_repository::FilesRepositoryImpl;
use crate::files::services::files_service::FilesServiceImpl;
use crate::roles::repositories::roles_repository::RolesRepositoryImpl;
use crate::roles::repositories::user_roles_repository::UserRolesRepositoryImpl;
use crate::roles::services::roles_service::RolesServiceImpl;
use crate::roles::services::user_roles_service::UserRolesServiceImpl;
use crate::users::repositories::users_repository::UsersRepositoryImpl;
use crate::users::services::users_service::UsersServiceImpl;
use sea_orm::DatabaseConnection;
use shaku::{Component, HasComponent, Interface, Module, Provider, module};
use std::path::Path;
use std::sync::Arc;
use tera::Tera;

// Connection pool interfaces
pub trait IDatabaseConnection: Interface {
	fn get_connection(&self) -> &DatabaseConnection;
}

// Connection pool implementation
#[derive(Component)]
#[shaku(interface = IDatabaseConnection)]
pub struct DatabaseConnectionImpl {
	connection: Arc<DatabaseConnection>,
}

impl DatabaseConnectionImpl {
	pub fn new(connection: Arc<DatabaseConnection>) -> Self {
		Self { connection }
	}
}

impl IDatabaseConnection for DatabaseConnectionImpl {
	fn get_connection(&self) -> &DatabaseConnection {
		&self.connection
	}
}

// Configuration interface
pub trait IAppConfig: Interface {
	fn get_config(&self) -> &AppConfig;
}

// Configuration implementation
#[derive(Component)]
#[shaku(interface = IAppConfig)]
pub struct AppConfigImpl {
	config: Arc<AppConfig>,
}

impl AppConfigImpl {
	pub fn new(config: Arc<AppConfig>) -> Self {
		Self { config }
	}
}

impl IAppConfig for AppConfigImpl {
	fn get_config(&self) -> &AppConfig {
		&self.config
	}
}

pub trait ITemplates: Interface {
	fn get_templates(&self) -> &Tera;
}

#[derive(Component)]
#[shaku(interface = ITemplates)]
pub struct TemplatesImpl {
	templates: Arc<Tera>,
}

impl ITemplates for TemplatesImpl {
	fn get_templates(&self) -> &Tera {
		&self.templates
	}
}

module! {
	pub AppModule {
		components = [
			// Connection pools and configuration
			DatabaseConnectionImpl,
			AppConfigImpl,

			// Repositories and Services
			UsersRepositoryImpl,
			RolesRepositoryImpl,
			UserRolesRepositoryImpl,
			RefreshTokenRepositoryImpl,
			FilesRepositoryImpl,
			EmailsServiceImpl,
			ConfirmationTokenServiceImpl,
			RefreshTokenServiceImpl,
			RolesServiceImpl,
			UserRolesServiceImpl,
			UsersServiceImpl,
			AuthServiceImpl,
			FilesServiceImpl,
			SmtpEmailStrategyImpl,

			// Custom components
			TemplatesImpl,
		],
		providers = [ ]
	}
}

pub fn initialize_di(db_connection: Arc<DatabaseConnection>, app_config: Arc<AppConfig>) -> Arc<AppModule> {
	let templates_path = Path::new(&app_config.emails.email_templates_dir).join("**/*.html");
	let templates = Tera::new(templates_path.to_str().unwrap()).unwrap_or_else(|e| {
		tracing::error!("Error parsing templates: {}", e);
		Tera::default()
	});

	let module = AppModule::builder()
		.with_component_parameters::<DatabaseConnectionImpl>(DatabaseConnectionImplParameters {
			connection: db_connection,
		})
		.with_component_parameters::<AppConfigImpl>(AppConfigImplParameters { config: app_config })
		.with_component_parameters::<TemplatesImpl>(TemplatesImplParameters {
			templates: Arc::new(templates),
		})
		.build();

	Arc::new(module)
}
