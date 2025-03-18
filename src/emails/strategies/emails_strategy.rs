use crate::common::enums::environment_enum::EnvironmentEnum;
use crate::common::error::app_error::AppError;
use crate::config::app_config::AppConfig;
use crate::di::IAppConfig;
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::Client;
use serde_json::json;
use shaku::{Component, Interface};
use std::sync::Arc;

#[async_trait]
pub trait IEmailStrategy: Interface {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError>;
}

#[derive(Component)]
#[shaku(interface = IEmailStrategy)]
pub struct MailDevEmailStrategyImpl {
	pub base_url: String,
	pub from_email: String,
}

impl MailDevEmailStrategyImpl {
	pub fn new(base_url: String, from_email: String) -> Self {
		Self { base_url, from_email }
	}
}

#[derive(Component)]
#[shaku(interface = IEmailStrategy)]
pub struct SmtpEmailStrategyImpl {
	#[shaku(inject)]
	app_config: Arc<dyn IAppConfig>,
}

impl SmtpEmailStrategyImpl {
	pub fn new(app_config: Arc<dyn IAppConfig>) -> Self {
		let config = app_config.get_config();
		Self {
			app_config: app_config.clone(),
		}
	}
}

#[async_trait]
impl IEmailStrategy for SmtpEmailStrategyImpl {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
		let email = Message::builder()
			.from(self.app_config.get_config().emails.email_from.parse().map_err(|_| {
				tracing::error!("Invalid from email: {}", self.app_config.get_config().emails.email_from);
				AppError::InternalError
			})?)
			.to(to.parse().map_err(|_| {
				tracing::error!("Invalid to email: {}", to);
				AppError::InternalError
			})?)
			.subject(subject)
			.header(lettre::message::header::ContentType::TEXT_HTML)
			.body(body.to_string())
			.map_err(|e| {
				tracing::error!("Failed to build email message: {}", e);
				AppError::InternalError
			})?;

		let mut builder = SmtpTransport::builder_dangerous(&self.app_config.get_config().emails.smtp_host)
			.port(self.app_config.get_config().emails.smtp_port);

		if !self.app_config.get_config().emails.smtp_username.is_empty()
			&& !self.app_config.get_config().emails.smtp_password.is_empty()
		{
			let creds = Credentials::new(
				self.app_config.get_config().emails.smtp_username.clone(),
				self.app_config.get_config().emails.smtp_password.clone(),
			);
			builder = builder.credentials(creds);
		}

		builder = builder.timeout(Some(std::time::Duration::from_secs(30)));
		let mailer = builder.build();

		match mailer.send(&email) {
			Ok(_) => {
				tracing::info!("Email sent via SMTP to {}", to);
				Ok(())
			}
			Err(e) => {
				tracing::error!("Failed to send email via SMTP: {}", e);
				Err(AppError::InternalError)
			}
		}
	}
}

pub fn get_email_strategy(
	environment: &str,
	config: &crate::config::app_config::EmailsConfig,
	app_config: Arc<dyn IAppConfig>,
) -> Arc<dyn IEmailStrategy> {
	match EnvironmentEnum::from_str(environment) {
		Some(EnvironmentEnum::Development) => Arc::new(SmtpEmailStrategyImpl::new(app_config)),
		_ => Arc::new(SmtpEmailStrategyImpl::new(app_config)),
	}
}
#[async_trait]
impl IEmailStrategy for MailDevEmailStrategyImpl {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
		let client = Client::new();

		let payload = json!({
			"from": self.from_email,
			"to": to,
			"subject": subject,
			"html": body
		});

		match client
			.post(&format!("{}/email", self.base_url))
			.json(&payload)
			.send()
			.await
		{
			Ok(response) => {
				if response.status().is_success() {
					tracing::info!("Email sent via MailDev to {}", to);
					Ok(())
				} else {
					tracing::error!("Failed to send email via MailDev: HTTP {}", response.status());
					Err(AppError::InternalError)
				}
			}
			Err(e) => {
				tracing::error!("Failed to send email via MailDev: {}", e);
				Err(AppError::InternalError)
			}
		}
	}
}
