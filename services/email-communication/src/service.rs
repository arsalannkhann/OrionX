//! Email Service
//! 
//! Core email orchestration logic.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::smtp_client::SmtpClient;
use crate::template_engine::TemplateEngine;
use crate::{SendEmailRequest, SendEmailResponse, EmailResponse, TemplateInfo, RenderTemplateResponse};

/// Stored email record
#[derive(Debug, Clone)]
struct StoredEmail {
    id: Uuid,
    thread_id: String,
    supplier_id: Uuid,
    direction: String,
    subject: String,
    body: String,
    sent_at: Option<String>,
    received_at: Option<String>,
    delivery_status: String,
    processing_status: String,
}

/// Email service
#[derive(Clone)]
#[allow(dead_code)]
pub struct EmailService {
    emails: Arc<RwLock<HashMap<Uuid, StoredEmail>>>,
    template_engine: Arc<TemplateEngine>,
    smtp_client: Arc<SmtpClient>,
}

impl EmailService {
    pub fn new() -> Self {
        Self {
            emails: Arc::new(RwLock::new(HashMap::new())),
            template_engine: Arc::new(TemplateEngine::new()),
            smtp_client: Arc::new(SmtpClient::default()),
        }
    }
    
    /// Send compliance email
    pub async fn send_compliance_email(&self, request: SendEmailRequest) -> Result<SendEmailResponse> {
        // Convert string variables to JSON values
        let json_vars: HashMap<String, serde_json::Value> = request.variables.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        
        // Render template
        let rendered = self.template_engine.render(&request.template_id, &json_vars)
            .context("Failed to render template")?;
        
        let subject = request.subject.unwrap_or(rendered.subject.clone());
        
        // For now, simulate sending (actual SMTP requires configuration)
        let email_id = Uuid::new_v4();
        let thread_id = format!("thread_{}", email_id);
        let sent_at = chrono::Utc::now().to_rfc3339();
        
        // Store email record
        let email = StoredEmail {
            id: email_id,
            thread_id: thread_id.clone(),
            supplier_id: request.supplier_id,
            direction: "outbound".to_string(),
            subject: subject.clone(),
            body: rendered.body_html,
            sent_at: Some(sent_at.clone()),
            received_at: None,
            delivery_status: "sent".to_string(),
            processing_status: "complete".to_string(),
        };
        
        let mut emails = self.emails.write().await;
        emails.insert(email_id, email);
        
        Ok(SendEmailResponse {
            email_id,
            thread_id,
            recipient: request.variables.get("contact_email").cloned().unwrap_or_default(),
            subject,
            status: "sent".to_string(),
            sent_at,
        })
    }
    
    /// Get email by ID
    pub async fn get_email(&self, id: Uuid) -> Result<Option<EmailResponse>> {
        let emails = self.emails.read().await;
        Ok(emails.get(&id).map(|e| self.to_response(e)))
    }
    
    /// Get emails in thread
    pub async fn get_thread(&self, thread_id: &str) -> Result<Vec<EmailResponse>> {
        let emails = self.emails.read().await;
        Ok(emails.values()
            .filter(|e| e.thread_id == thread_id)
            .map(|e| self.to_response(e))
            .collect())
    }
    
    /// Get emails for supplier
    pub async fn get_supplier_emails(&self, supplier_id: Uuid) -> Result<Vec<EmailResponse>> {
        let emails = self.emails.read().await;
        Ok(emails.values()
            .filter(|e| e.supplier_id == supplier_id)
            .map(|e| self.to_response(e))
            .collect())
    }
    
    /// List available templates
    pub fn list_templates(&self) -> Vec<TemplateInfo> {
        self.template_engine.list_templates().iter()
            .map(|t| TemplateInfo {
                id: t.id.clone(),
                name: t.name.clone(),
                description: t.description.clone(),
                variables: t.variables.iter().map(|v| v.name.clone()).collect(),
            })
            .collect()
    }
    
    /// Render template preview
    pub fn render_template(&self, template_id: &str, variables: &HashMap<String, String>) -> Result<RenderTemplateResponse> {
        let json_vars: HashMap<String, serde_json::Value> = variables.iter()
            .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
            .collect();
        
        let rendered = self.template_engine.render(template_id, &json_vars)?;
        
        Ok(RenderTemplateResponse {
            subject: rendered.subject,
            body: rendered.body_html,
        })
    }
    
    fn to_response(&self, email: &StoredEmail) -> EmailResponse {
        EmailResponse {
            id: email.id,
            thread_id: email.thread_id.clone(),
            supplier_id: email.supplier_id,
            direction: email.direction.clone(),
            subject: email.subject.clone(),
            body: email.body.clone(),
            sent_at: email.sent_at.clone(),
            received_at: email.received_at.clone(),
            delivery_status: email.delivery_status.clone(),
            processing_status: email.processing_status.clone(),
        }
    }
}

impl Default for EmailService {
    fn default() -> Self {
        Self::new()
    }
}
