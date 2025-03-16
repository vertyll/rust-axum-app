use crate::common::enums::environment_enum::EnvironmentEnum;
use crate::common::error::app_error::AppError;
use async_trait::async_trait;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use reqwest::Client;
use serde_json::json;
use std::sync::Arc;

#[async_trait]
pub trait EmailStrategy: Send + Sync {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError>;
}

pub struct MailDevEmailStrategy {
	pub base_url: String,
	pub from_email: String,
}

impl MailDevEmailStrategy {
	pub fn new(base_url: String, from_email: String) -> Self {
		Self { base_url, from_email }
	}
}

pub struct SmtpEmailStrategy {
	pub smtp_host: String,
	pub smtp_port: u16,
	pub smtp_username: String,
	pub smtp_password: String,
	pub from_email: String,
}

impl SmtpEmailStrategy {
	pub fn new(
		smtp_host: String,
		smtp_port: u16,
		smtp_username: String,
		smtp_password: String,
		from_email: String,
	) -> Self {
		Self {
			smtp_host,
			smtp_port,
			smtp_username,
			smtp_password,
			from_email,
		}
	}
}

#[async_trait]
impl EmailStrategy for SmtpEmailStrategy {
	async fn send_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
		let email = Message::builder()
			.from(self.from_email.parse().map_err(|_| {
				tracing::error!("Invalid from email: {}", self.from_email);
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

		let mut builder = SmtpTransport::builder_dangerous(&self.smtp_host).port(self.smtp_port);

		if !self.smtp_username.is_empty() && !self.smtp_password.is_empty() {
			let creds = Credentials::new(self.smtp_username.clone(), self.smtp_password.clone());
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
				tracing::debug!(
					"SMTP connection details: host={}, port={}, username={}",
					self.smtp_host,
					self.smtp_port,
					self.smtp_username
				);
				Err(AppError::InternalError)
			}
		}
	}
}

pub fn get_email_strategy(
	environment: &str,
	config: &crate::config::app_config::EmailsConfig,
) -> Arc<dyn EmailStrategy> {
	match EnvironmentEnum::from_str(environment) {
		Some(EnvironmentEnum::Development) => Arc::new(SmtpEmailStrategy::new(
			config.smtp_host.clone(),
			config.smtp_port.clone(),
			config.smtp_username.clone(),
			config.smtp_password.clone(),
			config.email_from.clone(),
		)),
		_ => Arc::new(SmtpEmailStrategy::new(
			config.smtp_host.clone(),
			config.smtp_port,
			config.smtp_username.clone(),
			config.smtp_password.clone(),
			config.email_from.clone(),
		)),
	}
}
