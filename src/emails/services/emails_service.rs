use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::config::app_config::AppConfig;
use crate::di::{IAppConfig, ITemplates};
use crate::emails::strategies::emails_strategy::IEmailStrategy;
use async_trait::async_trait;
use shaku::{Component, Interface};
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Component)]
#[shaku(interface = IEmailsService)]
pub struct EmailsServiceImpl {
	#[shaku(inject)]
	pub email_strategy: Arc<dyn IEmailStrategy>,
	#[shaku(inject)]
	pub app_config: Arc<dyn IAppConfig>,
	#[shaku(inject)]
	pub templates: Arc<dyn ITemplates>,
}

impl EmailsServiceImpl {
	fn render_template(&self, template_name: &str, context: &Context) -> Result<String, AppError> {
		self.templates
			.get_templates()
			.render(template_name, context)
			.map_err(|e| {
				tracing::error!("Template rendering error: {}", e);
				AppError::InternalError
			})
	}
}

#[async_trait]
pub trait IEmailsService: Interface {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError>;
	async fn send_email_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
	async fn send_password_reset(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
	async fn send_email_change_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
}

#[async_trait]
impl IEmailsService for EmailsServiceImpl {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
		self.email_strategy.send_email(to, subject, body).await
	}

	async fn send_email_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"confirmation_link",
			&format!(
				"{}/api/auth/confirm-email?token={}",
				self.app_config.get_config().server.app_url,
				token
			),
		);

		let body = self.render_template("email_confirmation.html", &context)?;
		self.email_strategy.send_email(to, "Confirm Your Email", &body).await
	}

	async fn send_password_reset(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"reset_link",
			&format!(
				"{}/api/auth/confirm-password-reset?token={}",
				self.app_config.get_config().server.app_url,
				token
			),
		);

		let body = self.render_template("password_reset.html", &context)?;
		self.email_strategy.send_email(to, "Reset Your Password", &body).await
	}

	async fn send_email_change_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"confirmation_link",
			&format!(
				"{}/api/auth/confirm-email-change?token={}",
				self.app_config.get_config().server.app_url,
				token
			),
		);

		let body = self.render_template("email_change.html", &context)?;
		self.email_strategy.send_email(to, "Confirm Email Change", &body).await
	}
}
