//! Elementa Email Communication Service
//! 
//! Handles bidirectional email communication with suppliers.
//! Features SMTP sending, IMAP response processing, and template generation.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

mod smtp_client;
mod template_engine;
mod service;

use service::EmailService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Elementa Email Communication Service");
    
    let service = EmailService::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/emails/send", post(send_email))
        .route("/api/v1/emails/:id", get(get_email))
        .route("/api/v1/emails/thread/:thread_id", get(get_thread))
        .route("/api/v1/emails/supplier/:supplier_id", get(get_supplier_emails))
        .route("/api/v1/templates", get(list_templates))
        .route("/api/v1/templates/:template_id/render", post(render_template))
        .layer(TraceLayer::new_for_http())
        .with_state(service);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8084));
    let listener = TcpListener::bind(&addr).await?;
    info!("Email Communication Service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "email-communication",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Send email request
#[derive(Debug, Deserialize)]
pub struct SendEmailRequest {
    pub supplier_id: Uuid,
    pub template_id: String,
    pub subject: Option<String>,
    pub variables: std::collections::HashMap<String, String>,
    pub attachments: Option<Vec<AttachmentRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct AttachmentRequest {
    pub filename: String,
    pub content_base64: String,
}

/// Send email response
#[derive(Debug, Serialize)]
pub struct SendEmailResponse {
    pub email_id: Uuid,
    pub thread_id: String,
    pub recipient: String,
    pub subject: String,
    pub status: String,
    pub sent_at: String,
}

async fn send_email(
    State(service): State<EmailService>,
    Json(request): Json<SendEmailRequest>,
) -> Result<Json<SendEmailResponse>, (StatusCode, String)> {
    let result = service.send_compliance_email(request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(result))
}

/// Email response
#[derive(Debug, Serialize)]
pub struct EmailResponse {
    pub id: Uuid,
    pub thread_id: String,
    pub supplier_id: Uuid,
    pub direction: String,
    pub subject: String,
    pub body: String,
    pub sent_at: Option<String>,
    pub received_at: Option<String>,
    pub delivery_status: String,
    pub processing_status: String,
}

async fn get_email(
    State(service): State<EmailService>,
    Path(id): Path<Uuid>,
) -> Result<Json<EmailResponse>, (StatusCode, String)> {
    let email = service.get_email(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Email not found".to_string()))?;
    
    Ok(Json(email))
}

async fn get_thread(
    State(service): State<EmailService>,
    Path(thread_id): Path<String>,
) -> Result<Json<Vec<EmailResponse>>, (StatusCode, String)> {
    let emails = service.get_thread(&thread_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(emails))
}

async fn get_supplier_emails(
    State(service): State<EmailService>,
    Path(supplier_id): Path<Uuid>,
) -> Result<Json<Vec<EmailResponse>>, (StatusCode, String)> {
    let emails = service.get_supplier_emails(supplier_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(emails))
}

/// Template list response
#[derive(Debug, Serialize)]
pub struct TemplateListResponse {
    pub templates: Vec<TemplateInfo>,
}

#[derive(Debug, Serialize)]
pub struct TemplateInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub variables: Vec<String>,
}

async fn list_templates(
    State(service): State<EmailService>,
) -> Json<TemplateListResponse> {
    let templates = service.list_templates();
    Json(TemplateListResponse { templates })
}

/// Render template request
#[derive(Debug, Deserialize)]
pub struct RenderTemplateRequest {
    pub variables: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize)]
pub struct RenderTemplateResponse {
    pub subject: String,
    pub body: String,
}

async fn render_template(
    State(service): State<EmailService>,
    Path(template_id): Path<String>,
    Json(request): Json<RenderTemplateRequest>,
) -> Result<Json<RenderTemplateResponse>, (StatusCode, String)> {
    let result = service.render_template(&template_id, &request.variables)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    
    Ok(Json(result))
}