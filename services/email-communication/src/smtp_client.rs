//! SMTP Client
//! 
//! Handles email sending via SMTP using lettre.

use anyhow::{Context, Result};
use lettre::{
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};

/// SMTP client configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from_email: String,
    pub from_name: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| "smtp.example.com".to_string()),
            port: std::env::var("SMTP_PORT").unwrap_or_else(|_| "587".to_string()).parse().unwrap_or(587),
            username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            from_email: std::env::var("SMTP_FROM_EMAIL").unwrap_or_else(|_| "compliance@elementa.io".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Elementa Compliance".to_string()),
        }
    }
}

/// SMTP client for sending emails
#[allow(dead_code)]
pub struct SmtpClient {
    config: SmtpConfig,
}

impl SmtpClient {
    pub fn new(config: SmtpConfig) -> Self {
        Self { config }
    }
    
    /// Send email
    #[allow(dead_code)]
    pub async fn send(&self, to_email: &str, to_name: &str, subject: &str, body_html: &str, body_text: &str) -> Result<String> {
        let from_mailbox: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .context("Invalid from address")?;
        
        let to_mailbox: Mailbox = format!("{} <{}>", to_name, to_email)
            .parse()
            .context("Invalid to address")?;
        
        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(subject)
            .multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body_text.to_string())
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(body_html.to_string())
                    )
            )
            .context("Failed to build email")?;
        
        let creds = Credentials::new(
            self.config.username.clone(),
            self.config.password.clone(),
        );
        
        let mailer: AsyncSmtpTransport<Tokio1Executor> = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.host)
            .context("Failed to create SMTP transport")?
            .port(self.config.port)
            .credentials(creds)
            .build();
        
        let response = mailer.send(email).await
            .context("Failed to send email")?;
        
        Ok(response.message().collect::<Vec<_>>().join("\n"))
    }
}

impl Default for SmtpClient {
    fn default() -> Self {
        Self::new(SmtpConfig::default())
    }
}
