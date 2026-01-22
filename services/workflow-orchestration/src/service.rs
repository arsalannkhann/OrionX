//! Workflow Service
//! 
//! Core workflow orchestration logic.

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::state_machine::{WorkflowState, TaskState, TaskType};
use crate::scheduler::WorkflowScheduler;
use crate::{
    CreateWorkflowRequest, WorkflowConfig, WorkflowResponse, WorkflowProgress,
    TaskResponse, EscalationResponse,
};

/// Stored workflow
#[derive(Debug, Clone)]
struct StoredWorkflow {
    id: Uuid,
    client_id: Uuid,
    campaign_name: String,
    suppliers: Vec<Uuid>,
    state: WorkflowState,
    #[allow(dead_code)]
    config: WorkflowConfig,
    start_date: DateTime<Utc>,
    deadline: DateTime<Utc>,
    progress: WorkflowProgress,
}

/// Stored task
#[derive(Debug, Clone)]
struct StoredTask {
    id: Uuid,
    workflow_id: Uuid,
    supplier_id: Uuid,
    task_type: TaskType,
    state: TaskState,
    retry_count: i32,
    max_retries: i32,
    scheduled_at: Option<DateTime<Utc>>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
    error: Option<String>,
    result: Option<serde_json::Value>,
}

/// Stored escalation
#[derive(Debug, Clone)]
struct StoredEscalation {
    id: Uuid,
    workflow_id: Uuid,
    supplier_id: Uuid,
    reason: String,
    severity: String,
    created_at: DateTime<Utc>,
    resolved: bool,
    resolved_at: Option<DateTime<Utc>>,
    resolution: Option<String>,
}

/// Workflow service
#[derive(Clone)]
pub struct WorkflowService {
    workflows: Arc<RwLock<HashMap<Uuid, StoredWorkflow>>>,
    tasks: Arc<RwLock<HashMap<Uuid, StoredTask>>>,
    escalations: Arc<RwLock<HashMap<Uuid, StoredEscalation>>>,
    #[allow(dead_code)]
    scheduler: Arc<WorkflowScheduler>,
}

impl WorkflowService {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            escalations: Arc::new(RwLock::new(HashMap::new())),
            scheduler: Arc::new(WorkflowScheduler::default()),
        }
    }
    
    /// Create new workflow
    pub async fn create_workflow(&self, request: CreateWorkflowRequest) -> Result<WorkflowResponse> {
        let config = request.config.unwrap_or_default();
        let deadline = DateTime::parse_from_rfc3339(&request.deadline)
            .context("Invalid deadline format")?
            .with_timezone(&Utc);
        
        let workflow = StoredWorkflow {
            id: Uuid::new_v4(),
            client_id: request.client_id,
            campaign_name: request.campaign_name,
            suppliers: request.supplier_ids.clone(),
            state: WorkflowState::Active,
            config: config.clone(),
            start_date: Utc::now(),
            deadline,
            progress: WorkflowProgress {
                total_suppliers: request.supplier_ids.len(),
                contacted: 0,
                responded: 0,
                complete: 0,
                escalated: 0,
                percent_complete: 0.0,
            },
        };
        
        // Schedule initial outreach tasks
        let scheduler = WorkflowScheduler::new(config);
        let scheduled_tasks = scheduler.schedule_initial_outreach(workflow.id, &request.supplier_ids);
        
        // Store tasks
        let mut tasks_map = self.tasks.write().await;
        for st in scheduled_tasks {
            let task = StoredTask {
                id: st.id,
                workflow_id: st.workflow_id,
                supplier_id: st.supplier_id,
                task_type: st.task_type,
                state: TaskState::Scheduled,
                retry_count: 0,
                max_retries: 3,
                scheduled_at: Some(st.scheduled_at),
                started_at: None,
                completed_at: None,
                error: None,
                result: None,
            };
            tasks_map.insert(task.id, task);
        }
        drop(tasks_map);
        
        let task_count = request.supplier_ids.len();
        
        // Store workflow
        let mut workflows = self.workflows.write().await;
        workflows.insert(workflow.id, workflow.clone());
        
        Ok(self.to_workflow_response(&workflow, task_count))
    }
    
    /// List all workflows
    pub async fn list_workflows(&self) -> Result<Vec<WorkflowResponse>> {
        let workflows = self.workflows.read().await;
        let tasks = self.tasks.read().await;
        
        Ok(workflows.values().map(|w| {
            let task_count = tasks.values().filter(|t| t.workflow_id == w.id).count();
            self.to_workflow_response(w, task_count)
        }).collect())
    }
    
    /// Get workflow by ID
    pub async fn get_workflow(&self, id: Uuid) -> Result<Option<WorkflowResponse>> {
        let workflows = self.workflows.read().await;
        let tasks = self.tasks.read().await;
        
        Ok(workflows.get(&id).map(|w| {
            let task_count = tasks.values().filter(|t| t.workflow_id == w.id).count();
            self.to_workflow_response(w, task_count)
        }))
    }
    
    /// Update workflow status
    pub async fn update_status(&self, id: Uuid, status: &str) -> Result<WorkflowResponse> {
        let new_state = WorkflowState::from_str(status)
            .context("Invalid status")?;
        
        let mut workflows = self.workflows.write().await;
        let workflow = workflows.get_mut(&id)
            .context("Workflow not found")?;
        
        if !workflow.state.can_transition_to(new_state) {
            bail!("Invalid state transition from {} to {}", workflow.state, new_state);
        }
        
        workflow.state = new_state;
        
        let tasks = self.tasks.read().await;
        let task_count = tasks.values().filter(|t| t.workflow_id == id).count();
        
        Ok(self.to_workflow_response(workflow, task_count))
    }
    
    /// Cancel workflow
    pub async fn cancel_workflow(&self, id: Uuid) -> Result<WorkflowResponse> {
        self.update_status(id, "cancelled").await
    }
    
    /// Get tasks for workflow
    pub async fn get_workflow_tasks(&self, workflow_id: Uuid) -> Result<Vec<TaskResponse>> {
        let tasks = self.tasks.read().await;
        
        Ok(tasks.values()
            .filter(|t| t.workflow_id == workflow_id)
            .map(|t| self.to_task_response(t))
            .collect())
    }
    
    /// Get task by ID
    pub async fn get_task(&self, task_id: Uuid) -> Result<Option<TaskResponse>> {
        let tasks = self.tasks.read().await;
        Ok(tasks.get(&task_id).map(|t| self.to_task_response(t)))
    }
    
    /// Complete task
    pub async fn complete_task(&self, task_id: Uuid, result: Option<serde_json::Value>) -> Result<TaskResponse> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(&task_id)
            .context("Task not found")?;
        
        task.state = TaskState::Completed;
        task.completed_at = Some(Utc::now());
        task.result = result;
        
        // Update workflow progress
        self.update_workflow_progress(task.workflow_id).await;
        
        Ok(self.to_task_response(task))
    }
    
    /// Retry task
    pub async fn retry_task(&self, task_id: Uuid) -> Result<TaskResponse> {
        let mut tasks = self.tasks.write().await;
        let task = tasks.get_mut(&task_id)
            .context("Task not found")?;
        
        if task.retry_count >= task.max_retries {
            task.state = TaskState::Exhausted;
            
            // Create escalation
            self.create_escalation(
                task.workflow_id,
                task.supplier_id,
                "Max retries exceeded".to_string(),
                "high".to_string(),
            ).await?;
        } else {
            task.retry_count += 1;
            task.state = TaskState::Scheduled;
            task.scheduled_at = Some(Utc::now());
            task.error = None;
        }
        
        Ok(self.to_task_response(task))
    }
    
    /// List escalations
    pub async fn list_escalations(&self) -> Result<Vec<EscalationResponse>> {
        let escalations = self.escalations.read().await;
        Ok(escalations.values().map(|e| self.to_escalation_response(e)).collect())
    }
    
    /// Resolve escalation
    pub async fn resolve_escalation(&self, id: Uuid, resolution: &str) -> Result<EscalationResponse> {
        let mut escalations = self.escalations.write().await;
        let escalation = escalations.get_mut(&id)
            .context("Escalation not found")?;
        
        escalation.resolved = true;
        escalation.resolved_at = Some(Utc::now());
        escalation.resolution = Some(resolution.to_string());
        
        Ok(self.to_escalation_response(escalation))
    }
    
    /// Create escalation (internal)
    async fn create_escalation(&self, workflow_id: Uuid, supplier_id: Uuid, reason: String, severity: String) -> Result<()> {
        let escalation = StoredEscalation {
            id: Uuid::new_v4(),
            workflow_id,
            supplier_id,
            reason,
            severity,
            created_at: Utc::now(),
            resolved: false,
            resolved_at: None,
            resolution: None,
        };
        
        let mut escalations = self.escalations.write().await;
        escalations.insert(escalation.id, escalation);
        
        Ok(())
    }
    
    /// Update workflow progress (internal)
    async fn update_workflow_progress(&self, workflow_id: Uuid) {
        let tasks = self.tasks.read().await;
        let workflow_tasks: Vec<_> = tasks.values()
            .filter(|t| t.workflow_id == workflow_id)
            .collect();
        
        let total = workflow_tasks.len();
        let completed = workflow_tasks.iter().filter(|t| t.state == TaskState::Completed).count();
        
        drop(tasks);
        
        let mut workflows = self.workflows.write().await;
        if let Some(workflow) = workflows.get_mut(&workflow_id) {
            workflow.progress.complete = completed;
            workflow.progress.percent_complete = if total > 0 {
                (completed as f64 / total as f64) * 100.0
            } else {
                0.0
            };
            
            // Check if workflow is complete
            if completed == total && total > 0 {
                workflow.state = WorkflowState::Completed;
            }
        }
    }
    
    fn to_workflow_response(&self, w: &StoredWorkflow, task_count: usize) -> WorkflowResponse {
        WorkflowResponse {
            id: w.id,
            client_id: w.client_id,
            campaign_name: w.campaign_name.clone(),
            status: w.state.to_string(),
            start_date: w.start_date.to_rfc3339(),
            deadline: w.deadline.to_rfc3339(),
            progress: w.progress.clone(),
            supplier_count: w.suppliers.len(),
            task_count,
        }
    }
    
    fn to_task_response(&self, t: &StoredTask) -> TaskResponse {
        TaskResponse {
            id: t.id,
            workflow_id: t.workflow_id,
            task_type: t.task_type.to_string(),
            supplier_id: t.supplier_id,
            status: t.state.to_string(),
            retry_count: t.retry_count,
            max_retries: t.max_retries,
            scheduled_at: t.scheduled_at.map(|d| d.to_rfc3339()),
            started_at: t.started_at.map(|d| d.to_rfc3339()),
            completed_at: t.completed_at.map(|d| d.to_rfc3339()),
            error: t.error.clone(),
        }
    }
    
    fn to_escalation_response(&self, e: &StoredEscalation) -> EscalationResponse {
        EscalationResponse {
            id: e.id,
            workflow_id: e.workflow_id,
            supplier_id: e.supplier_id,
            reason: e.reason.clone(),
            severity: e.severity.clone(),
            created_at: e.created_at.to_rfc3339(),
            resolved: e.resolved,
            resolved_at: e.resolved_at.map(|d| d.to_rfc3339()),
            resolution: e.resolution.clone(),
        }
    }
}

impl Default for WorkflowService {
    fn default() -> Self {
        Self::new()
    }
}
