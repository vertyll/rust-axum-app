//! Async SMTP + Tera 2 adapter for `IdentityMailer`. Templates load once at
//! startup; broken templates fail loudly instead of an empty fallback set.
//!
//! Uses lettre's *async* transport — the previous implementation called the
//! blocking `SmtpTransport` from async handlers, freezing a runtime worker
//! for the whole SMTP round-trip. Templates are rendered with Tera 2
//! (loaded once at startup; startup now fails loudly on broken templates
//! instead of silently falling back to an empty template set).

use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tera::{Context, Tera};

use crate::identity::application::ports::IdentityMailer;
use crate::identity::domain::{Email, IdentityError, Username};

#[derive(Clone)]
pub struct SmtpIdentityMailer {
	transport: AsyncSmtpTransport<Tokio1Executor>,
	from: Mailbox,
	templates: Arc<Tera>,
	app_url: String,
}

pub struct SmtpSettings<'a> {
	pub host: &'a str,
	pub port: u16,
	pub username: &'a str,
	pub password: &'a str,
	pub from: &'a str,
	pub templates_dir: &'a str,
	pub app_url: &'a str,
}

impl SmtpIdentityMailer {
	pub fn new(settings: SmtpSettings<'_>) -> anyhow::Result<Self> {
		let mut builder =
			AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(settings.host)
				.port(settings.port)
				.timeout(Some(Duration::from_secs(30)));

		if !settings.username.is_empty() && !settings.password.is_empty() {
			builder = builder.credentials(Credentials::new(
				settings.username.to_string(),
				settings.password.to_string(),
			));
		}

		let from = settings
			.from
			.parse::<Mailbox>()
			.with_context(|| format!("invalid EMAIL_FROM address: {}", settings.from))?;

		let mut templates = Tera::new();
		templates
			.load_from_glob(&format!("{}/**/*.html", settings.templates_dir))
			.with_context(|| {
				format!("failed to load e-mail templates from {}", settings.templates_dir)
			})?;

		Ok(Self {
			transport: builder.build(),
			from,
			templates: Arc::new(templates),
			app_url: settings.app_url.trim_end_matches('/').to_string(),
		})
	}

	async fn send(
		&self,
		to: &Email,
		subject: &str,
		template: &str,
		context: &Context,
	) -> Result<(), IdentityError> {
		let body = self.templates.render(template, context).map_err(|err| {
			tracing::error!("template rendering failed for {template}: {err}");
			IdentityError::MailerFailure(format!("template {template}"))
		})?;

		let message = Message::builder()
			.from(self.from.clone())
			.to(to
				.as_str()
				.parse()
				.map_err(|_| IdentityError::MailerFailure(format!("invalid recipient {to}")))?)
			.subject(subject)
			.header(ContentType::TEXT_HTML)
			.body(body)
			.map_err(|err| {
				tracing::error!("failed to build e-mail message: {err}");
				IdentityError::MailerFailure("message build".to_string())
			})?;

		self.transport.send(message).await.map(|_| ()).map_err(|err| {
			tracing::error!("SMTP send failed: {err}");
			IdentityError::MailerFailure("smtp send".to_string())
		})
	}

	fn link_context(&self, username: &Username, key: &'static str, path: &str, token: &str) -> Context {
		let mut context = Context::new();
		context.insert("username", username.as_str());
		context.insert(key, &format!("{}{path}?token={token}", self.app_url));
		context
	}
}

impl IdentityMailer for SmtpIdentityMailer {
	async fn send_email_confirmation(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> Result<(), IdentityError> {
		let context =
			self.link_context(username, "confirmation_link", "/api/auth/confirm-email", token);
		self.send(to, "Confirm Your Email", "email_confirmation.html", &context).await
	}

	async fn send_password_reset(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> Result<(), IdentityError> {
		let context =
			self.link_context(username, "reset_link", "/api/auth/confirm-password-reset", token);
		self.send(to, "Reset Your Password", "password_reset.html", &context).await
	}

	async fn send_email_change_confirmation(
		&self,
		to: &Email,
		username: &Username,
		token: &str,
	) -> Result<(), IdentityError> {
		let context = self.link_context(
			username,
			"confirmation_link",
			"/api/auth/confirm-email-change",
			token,
		);
		self.send(to, "Confirm Email Change", "email_change.html", &context).await
	}
}
