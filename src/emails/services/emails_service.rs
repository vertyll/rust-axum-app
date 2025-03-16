use crate::common::error::app_error::AppError;
use crate::common::r#struct::app_state::AppState;
use crate::config::app_config::AppConfig;
use crate::emails::strategies::emails_strategy::EmailStrategy;
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;
use tera::{Context, Tera};

#[derive(Clone)]
pub struct EmailService {
	pub email_strategy: Arc<dyn EmailStrategy>,
	pub templates: Arc<Tera>,
	pub app_url: String,
}

impl EmailService {
	pub fn new(app_state: AppState) -> Self {
		let app_url = app_state.config.server.app_url.clone();

		let environment = app_state.config.server.app_environment.clone();
		let emails = app_state.config.emails.clone();

		let email_strategy = crate::emails::strategies::emails_strategy::get_email_strategy(&environment, &emails);

		let templates_path = Path::new(&emails.email_templates_dir).join("**/*.html");
		let templates = Tera::new(templates_path.to_str().unwrap()).unwrap_or_else(|e| {
			tracing::error!("Error parsing templates: {}", e);
			Tera::default()
		});

		Self {
			email_strategy,
			templates: Arc::new(templates),
			app_url,
		}
	}

	fn render_template(&self, template_name: &str, context: &Context) -> Result<String, AppError> {
		self.templates.render(template_name, context).map_err(|e| {
			tracing::error!("Template rendering error: {}", e);
			AppError::InternalError
		})
	}
}

#[async_trait]
pub trait EmailServiceTrait: Send + Sync {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError>;
	async fn send_email_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
	async fn send_password_reset(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
	async fn send_email_change_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError>;
}

#[async_trait]
impl EmailServiceTrait for EmailService {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
		self.email_strategy.send_email(to, subject, body).await
	}

	async fn send_email_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"confirmation_link",
			&format!("{}/api/auth/confirm-email?token={}", self.app_url, token),
		);

		let body = self.render_template("email_confirmation.html", &context)?;
		self.email_strategy.send_email(to, "Confirm Your Email", &body).await
	}

	async fn send_password_reset(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"reset_link",
			&format!("{}/api/auth/confirm-password-reset?token={}", self.app_url, token),
		);

		let body = self.render_template("password_reset.html", &context)?;
		self.email_strategy.send_email(to, "Reset Your Password", &body).await
	}

	async fn send_email_change_confirmation(&self, to: &str, username: &str, token: &str) -> Result<(), AppError> {
		let mut context = Context::new();
		context.insert("username", username);
		context.insert(
			"confirmation_link",
			&format!("{}/api/auth/confirm-email-change?token={}", self.app_url, token),
		);

		let body = self.render_template("email_change.html", &context)?;
		self.email_strategy.send_email(to, "Confirm Email Change", &body).await
	}
}
