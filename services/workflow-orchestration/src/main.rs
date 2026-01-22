//! Elementa Workflow Orchestration Service
//! 
//! Manages compliance campaign workflows with state machine execution,
//! task scheduling, follow-up logic, and escalation handling.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

mod state_machine;
mod scheduler;
mod service;

use service::WorkflowService;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Elementa Workflow Orchestration Service");
    
    let service = WorkflowService::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        // Workflow management
        .route("/api/v1/workflows", post(create_workflow))
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows/:id", get(get_workflow))
        .route("/api/v1/workflows/:id/status", put(update_workflow_status))
        .route("/api/v1/workflows/:id/cancel", post(cancel_workflow))
        // Task management
        .route("/api/v1/workflows/:id/tasks", get(get_workflow_tasks))
        .route("/api/v1/tasks/:task_id", get(get_task))
        .route("/api/v1/tasks/:task_id/complete", post(complete_task))
        .route("/api/v1/tasks/:task_id/retry", post(retry_task))
        // Escalations
        .route("/api/v1/escalations", get(list_escalations))
        .route("/api/v1/escalations/:id/resolve", post(resolve_escalation))
        .layer(TraceLayer::new_for_http())
        .with_state(service);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8085));
    let listener = TcpListener::bind(&addr).await?;
    info!("Workflow Orchestration Service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "workflow-orchestration",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// ===== Workflow Endpoints =====

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub client_id: Uuid,
    pub campaign_name: String,
    pub supplier_ids: Vec<Uuid>,
    pub deadline: String,
    pub config: Option<WorkflowConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkflowConfig {
    pub max_follow_ups: i32,
    pub follow_up_interval_days: i32,
    pub auto_escalate: bool,
    pub escalation_threshold_days: i32,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_follow_ups: 3,
            follow_up_interval_days: 7,
            auto_escalate: true,
            escalation_threshold_days: 21,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct WorkflowResponse {
    pub id: Uuid,
    pub client_id: Uuid,
    pub campaign_name: String,
    pub status: String,
    pub start_date: String,
    pub deadline: String,
    pub progress: WorkflowProgress,
    pub supplier_count: usize,
    pub task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub total_suppliers: usize,
    pub contacted: usize,
    pub responded: usize,
    pub complete: usize,
    pub escalated: usize,
    pub percent_complete: f64,
}

async fn create_workflow(
    State(service): State<WorkflowService>,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let workflow = service.create_workflow(request).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(workflow))
}

async fn list_workflows(
    State(service): State<WorkflowService>,
) -> Result<Json<Vec<WorkflowResponse>>, (StatusCode, String)> {
    let workflows = service.list_workflows().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(workflows))
}

async fn get_workflow(
    State(service): State<WorkflowService>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let workflow = service.get_workflow(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Workflow not found".to_string()))?;
    
    Ok(Json(workflow))
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    pub status: String,
}

async fn update_workflow_status(
    State(service): State<WorkflowService>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateStatusRequest>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let workflow = service.update_status(id, &request.status).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(workflow))
}

async fn cancel_workflow(
    State(service): State<WorkflowService>,
    Path(id): Path<Uuid>,
) -> Result<Json<WorkflowResponse>, (StatusCode, String)> {
    let workflow = service.cancel_workflow(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(workflow))
}

// ===== Task Endpoints =====

#[derive(Debug, Serialize)]
pub struct TaskResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_type: String,
    pub supplier_id: Uuid,
    pub status: String,
    pub retry_count: i32,
    pub max_retries: i32,
    pub scheduled_at: Option<String>,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

async fn get_workflow_tasks(
    State(service): State<WorkflowService>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<TaskResponse>>, (StatusCode, String)> {
    let tasks = service.get_workflow_tasks(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(tasks))
}

async fn get_task(
    State(service): State<WorkflowService>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, (StatusCode, String)> {
    let task = service.get_task(task_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Task not found".to_string()))?;
    
    Ok(Json(task))
}

#[derive(Debug, Deserialize)]
pub struct CompleteTaskRequest {
    pub result: Option<serde_json::Value>,
}

async fn complete_task(
    State(service): State<WorkflowService>,
    Path(task_id): Path<Uuid>,
    Json(request): Json<CompleteTaskRequest>,
) -> Result<Json<TaskResponse>, (StatusCode, String)> {
    let task = service.complete_task(task_id, request.result).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(task))
}

async fn retry_task(
    State(service): State<WorkflowService>,
    Path(task_id): Path<Uuid>,
) -> Result<Json<TaskResponse>, (StatusCode, String)> {
    let task = service.retry_task(task_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(task))
}

// ===== Escalation Endpoints =====

#[derive(Debug, Serialize)]
pub struct EscalationResponse {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub supplier_id: Uuid,
    pub reason: String,
    pub severity: String,
    pub created_at: String,
    pub resolved: bool,
    pub resolved_at: Option<String>,
    pub resolution: Option<String>,
}

async fn list_escalations(
    State(service): State<WorkflowService>,
) -> Result<Json<Vec<EscalationResponse>>, (StatusCode, String)> {
    let escalations = service.list_escalations().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(escalations))
}

#[derive(Debug, Deserialize)]
pub struct ResolveEscalationRequest {
    pub resolution: String,
}

async fn resolve_escalation(
    State(service): State<WorkflowService>,
    Path(id): Path<Uuid>,
    Json(request): Json<ResolveEscalationRequest>,
) -> Result<Json<EscalationResponse>, (StatusCode, String)> {
    let escalation = service.resolve_escalation(id, &request.resolution).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(escalation))
}