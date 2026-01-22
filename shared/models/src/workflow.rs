use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub client_id: Uuid,
    pub campaign_name: String,
    pub suppliers: Vec<Uuid>,
    pub status: WorkflowStatus,
    pub start_date: DateTime<Utc>,
    pub deadline: DateTime<Utc>,
    pub progress: WorkflowProgress,
    pub escalations: Vec<Escalation>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Created,
    InProgress,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub total_suppliers: u32,
    pub contacted_suppliers: u32,
    pub responded_suppliers: u32,
    pub compliant_suppliers: u32,
    pub non_compliant_suppliers: u32,
    pub escalated_suppliers: u32,
    pub completion_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub escalation_type: EscalationType,
    pub reason: String,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EscalationType {
    NoResponse,
    IncompleteData,
    DataQualityIssue,
    TechnicalProblem,
    SupplierDispute,
    DeadlineRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub task_type: AgentTaskType,
    pub supplier_id: Uuid,
    pub context: TaskContext,
    pub status: TaskStatus,
    pub retry_count: u32,
    pub max_retries: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentTaskType {
    InitialOutreach,
    DocumentProcessing,
    FollowUp,
    Validation,
    Escalation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub components: Vec<Uuid>,
    pub deadline: DateTime<Utc>,
    pub priority: TaskPriority,
    pub custom_instructions: Option<String>,
    pub previous_attempts: Vec<TaskAttempt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    NotStarted,
    Queued,
    InProgress,
    Completed,
    Failed,
    Cancelled,
    RequiresIntervention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAttempt {
    pub attempt_number: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: TaskResult,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    Success,
    PartialSuccess,
    Failed,
    RequiresRetry,
    RequiresEscalation,
}

impl Default for WorkflowInstance {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            client_id: Uuid::new_v4(),
            campaign_name: String::new(),
            suppliers: Vec::new(),
            status: WorkflowStatus::Created,
            start_date: Utc::now(),
            deadline: Utc::now(),
            progress: WorkflowProgress::default(),
            escalations: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl Default for WorkflowProgress {
    fn default() -> Self {
        Self {
            total_suppliers: 0,
            contacted_suppliers: 0,
            responded_suppliers: 0,
            compliant_suppliers: 0,
            non_compliant_suppliers: 0,
            escalated_suppliers: 0,
            completion_percentage: 0.0,
        }
    }
}