//! Elementa Audit Trail Service
//! 
//! Immutable audit logging with hash chain verification
//! for regulatory compliance and chain of custody.

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Elementa Audit Trail Service");
    
    let service = AuditService::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/audit", post(create_audit_entry))
        .route("/api/v1/audit", get(list_audit_entries))
        .route("/api/v1/audit/:id", get(get_audit_entry))
        .route("/api/v1/audit/entity/:entity_type/:entity_id", get(get_entity_audit_trail))
        .route("/api/v1/audit/verify", post(verify_chain))
        .route("/api/v1/audit/export", post(export_audit_trail))
        .layer(TraceLayer::new_for_http())
        .with_state(service);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8086));
    let listener = TcpListener::bind(&addr).await?;
    info!("Audit Trail Service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "audit-trail",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ===== Data Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<String>,
    pub details: serde_json::Value,
    pub source_document: Option<DocumentReference>,
    pub hash: String,
    pub previous_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Create,
    Read,
    Update,
    Delete,
    Extract,
    Validate,
    Send,
    Receive,
    Escalate,
    Approve,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentReference {
    pub document_id: Uuid,
    pub filename: String,
    pub hash: String,
}

// ===== API Types =====

#[derive(Debug, Deserialize)]
pub struct CreateAuditRequest {
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<String>,
    pub details: serde_json::Value,
    pub source_document: Option<DocumentReference>,
}

#[derive(Debug, Serialize)]
pub struct AuditEntryResponse {
    pub id: Uuid,
    pub timestamp: String,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<String>,
    pub details: serde_json::Value,
    pub source_document: Option<DocumentReference>,
    pub hash: String,
    pub previous_hash: Option<String>,
    pub chain_valid: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub action: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub page: Option<i32>,
    pub page_size: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct AuditListResponse {
    pub entries: Vec<AuditEntryResponse>,
    pub total: usize,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Deserialize)]
pub struct VerifyChainRequest {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyChainResponse {
    pub is_valid: bool,
    pub entries_verified: usize,
    pub first_entry: String,
    pub last_entry: String,
    pub broken_links: Vec<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct ExportRequest {
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub from: String,
    pub to: String,
    pub format: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExportResponse {
    pub export_id: Uuid,
    pub entry_count: usize,
    pub format: String,
    pub download_url: String,
}

// ===== Service =====

#[derive(Clone)]
pub struct AuditService {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
}

impl AuditService {
    pub fn new() -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
        }
    }
    
    fn parse_action(s: &str) -> AuditAction {
        match s.to_lowercase().as_str() {
            "create" => AuditAction::Create,
            "read" => AuditAction::Read,
            "update" => AuditAction::Update,
            "delete" => AuditAction::Delete,
            "extract" => AuditAction::Extract,
            "validate" => AuditAction::Validate,
            "send" => AuditAction::Send,
            "receive" => AuditAction::Receive,
            "escalate" => AuditAction::Escalate,
            "approve" => AuditAction::Approve,
            "reject" => AuditAction::Reject,
            _ => AuditAction::Read,
        }
    }
    
    fn calculate_hash(entry: &AuditEntry, previous_hash: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entry.id.to_string().as_bytes());
        hasher.update(entry.timestamp.to_rfc3339().as_bytes());
        hasher.update(format!("{:?}", entry.action).as_bytes());
        hasher.update(entry.entity_type.as_bytes());
        hasher.update(entry.entity_id.to_string().as_bytes());
        hasher.update(entry.details.to_string().as_bytes());
        
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }
    
    fn to_response(entry: &AuditEntry, chain_valid: bool) -> AuditEntryResponse {
        AuditEntryResponse {
            id: entry.id,
            timestamp: entry.timestamp.to_rfc3339(),
            action: format!("{:?}", entry.action),
            entity_type: entry.entity_type.clone(),
            entity_id: entry.entity_id,
            user_id: entry.user_id,
            agent_id: entry.agent_id.clone(),
            details: entry.details.clone(),
            source_document: entry.source_document.clone(),
            hash: entry.hash.clone(),
            previous_hash: entry.previous_hash.clone(),
            chain_valid,
        }
    }
}

impl Default for AuditService {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Handlers =====

async fn create_audit_entry(
    State(service): State<AuditService>,
    Json(request): Json<CreateAuditRequest>,
) -> Result<Json<AuditEntryResponse>, (StatusCode, String)> {
    let mut entries = service.entries.write().await;
    
    let previous_hash = entries.last().map(|e| e.hash.clone());
    
    let mut entry = AuditEntry {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        action: AuditService::parse_action(&request.action),
        entity_type: request.entity_type,
        entity_id: request.entity_id,
        user_id: request.user_id,
        agent_id: request.agent_id,
        details: request.details,
        source_document: request.source_document,
        hash: String::new(),
        previous_hash: previous_hash.clone(),
    };
    
    entry.hash = AuditService::calculate_hash(&entry, previous_hash.as_deref());
    
    entries.push(entry.clone());
    
    Ok(Json(AuditService::to_response(&entry, true)))
}

async fn list_audit_entries(
    State(service): State<AuditService>,
    Query(query): Query<AuditQuery>,
) -> Json<AuditListResponse> {
    let entries = service.entries.read().await;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);
    
    let filtered: Vec<_> = entries.iter()
        .filter(|e| {
            query.entity_type.as_ref().map_or(true, |t| &e.entity_type == t) &&
            query.entity_id.map_or(true, |id| e.entity_id == id) &&
            query.action.as_ref().map_or(true, |a| format!("{:?}", e.action).to_lowercase() == a.to_lowercase())
        })
        .cloned()
        .collect();
    
    let total = filtered.len();
    let start = ((page - 1) * page_size) as usize;
    let end = (start + page_size as usize).min(total);
    
    let page_entries: Vec<_> = filtered[start..end].iter()
        .map(|e| AuditService::to_response(e, true))
        .collect();
    
    Json(AuditListResponse {
        entries: page_entries,
        total,
        page,
        page_size,
    })
}

async fn get_audit_entry(
    State(service): State<AuditService>,
    Path(id): Path<Uuid>,
) -> Result<Json<AuditEntryResponse>, (StatusCode, String)> {
    let entries = service.entries.read().await;
    
    entries.iter()
        .find(|e| e.id == id)
        .map(|e| Json(AuditService::to_response(e, true)))
        .ok_or((StatusCode::NOT_FOUND, "Audit entry not found".to_string()))
}

async fn get_entity_audit_trail(
    State(service): State<AuditService>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Json<Vec<AuditEntryResponse>> {
    let entries = service.entries.read().await;
    
    let trail: Vec<_> = entries.iter()
        .filter(|e| e.entity_type == entity_type && e.entity_id == entity_id)
        .map(|e| AuditService::to_response(e, true))
        .collect();
    
    Json(trail)
}

async fn verify_chain(
    State(service): State<AuditService>,
    Json(request): Json<VerifyChainRequest>,
) -> Result<Json<VerifyChainResponse>, (StatusCode, String)> {
    let from = DateTime::parse_from_rfc3339(&request.from)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid from date".to_string()))?
        .with_timezone(&Utc);
    
    let to = DateTime::parse_from_rfc3339(&request.to)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid to date".to_string()))?
        .with_timezone(&Utc);
    
    let entries = service.entries.read().await;
    
    let range_entries: Vec<_> = entries.iter()
        .filter(|e| e.timestamp >= from && e.timestamp <= to)
        .collect();
    
    let mut broken_links = Vec::new();
    let mut previous_hash: Option<String> = None;
    
    for entry in &range_entries {
        let expected_hash = AuditService::calculate_hash(entry, previous_hash.as_deref());
        
        if entry.hash != expected_hash {
            broken_links.push(entry.id);
        }
        
        previous_hash = Some(entry.hash.clone());
    }
    
    Ok(Json(VerifyChainResponse {
        is_valid: broken_links.is_empty(),
        entries_verified: range_entries.len(),
        first_entry: range_entries.first().map(|e| e.timestamp.to_rfc3339()).unwrap_or_default(),
        last_entry: range_entries.last().map(|e| e.timestamp.to_rfc3339()).unwrap_or_default(),
        broken_links,
    }))
}

async fn export_audit_trail(
    State(service): State<AuditService>,
    Json(request): Json<ExportRequest>,
) -> Result<Json<ExportResponse>, (StatusCode, String)> {
    let entries = service.entries.read().await;
    
    let from = DateTime::parse_from_rfc3339(&request.from)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid from date".to_string()))?
        .with_timezone(&Utc);
    
    let to = DateTime::parse_from_rfc3339(&request.to)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid to date".to_string()))?
        .with_timezone(&Utc);
    
    let filtered: Vec<_> = entries.iter()
        .filter(|e| {
            e.timestamp >= from && e.timestamp <= to &&
            request.entity_type.as_ref().map_or(true, |t| &e.entity_type == t) &&
            request.entity_id.map_or(true, |id| e.entity_id == id)
        })
        .collect();
    
    let export_id = Uuid::new_v4();
    let format = request.format.unwrap_or_else(|| "json".to_string());
    
    Ok(Json(ExportResponse {
        export_id,
        entry_count: filtered.len(),
        format: format.clone(),
        download_url: format!("/api/v1/audit/export/{}.{}", export_id, format),
    }))
}