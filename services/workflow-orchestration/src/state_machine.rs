//! Workflow State Machine
//! 
//! Defines workflow and task state transitions.

use serde::{Deserialize, Serialize};

/// Workflow states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowState {
    /// Workflow created but not started
    Pending,
    /// Workflow is actively running
    Active,
    /// Workflow is paused
    Paused,
    /// Workflow completed successfully
    Completed,
    /// Workflow was cancelled
    Cancelled,
    /// Workflow failed
    Failed,
}

#[allow(dead_code)]
impl WorkflowState {
    /// Check if transition is valid
    pub fn can_transition_to(&self, target: WorkflowState) -> bool {
        use WorkflowState::*;
        
        match (self, target) {
            // From Pending
            (Pending, Active) => true,
            (Pending, Cancelled) => true,
            
            // From Active
            (Active, Paused) => true,
            (Active, Completed) => true,
            (Active, Cancelled) => true,
            (Active, Failed) => true,
            
            // From Paused
            (Paused, Active) => true,
            (Paused, Cancelled) => true,
            
            // Terminal states cannot transition
            (Completed, _) => false,
            (Cancelled, _) => false,
            (Failed, _) => false,
            
            _ => false,
        }
    }
    
    /// Check if workflow is in terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkflowState::Completed | WorkflowState::Cancelled | WorkflowState::Failed)
    }
    
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(Self::Pending),
            "active" => Some(Self::Active),
            "paused" => Some(Self::Paused),
            "completed" => Some(Self::Completed),
            "cancelled" | "canceled" => Some(Self::Cancelled),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

impl std::fmt::Display for WorkflowState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::Active => write!(f, "active"),
            Self::Paused => write!(f, "paused"),
            Self::Completed => write!(f, "completed"),
            Self::Cancelled => write!(f, "cancelled"),
            Self::Failed => write!(f, "failed"),
        }
    }
}

/// Task states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    /// Task is scheduled but not started
    Scheduled,
    /// Task is currently executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed and may be retried
    Failed,
    /// Task failed after max retries
    Exhausted,
    /// Task was skipped
    Skipped,
    /// Task was cancelled
    Cancelled,
}

#[allow(dead_code)]
impl TaskState {
    pub fn can_transition_to(&self, target: TaskState) -> bool {
        use TaskState::*;
        
        match (self, target) {
            (Scheduled, Running) => true,
            (Scheduled, Skipped) => true,
            (Scheduled, Cancelled) => true,
            
            (Running, Completed) => true,
            (Running, Failed) => true,
            (Running, Cancelled) => true,
            
            (Failed, Running) => true, // Retry
            (Failed, Exhausted) => true,
            (Failed, Cancelled) => true,
            
            _ => false,
        }
    }
    
    pub fn is_terminal(&self) -> bool {
        matches!(self, TaskState::Completed | TaskState::Exhausted | TaskState::Skipped | TaskState::Cancelled)
    }
}

impl std::fmt::Display for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scheduled => write!(f, "scheduled"),
            Self::Running => write!(f, "running"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Exhausted => write!(f, "exhausted"),
            Self::Skipped => write!(f, "skipped"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

/// Task types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Initial compliance request outreach
    InitialOutreach,
    /// Process received documents
    DocumentProcessing,
    /// Follow-up for missing data
    FollowUp,
    /// Validate received compliance data
    Validation,
    /// Generate escalation
    Escalation,
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InitialOutreach => write!(f, "initial_outreach"),
            Self::DocumentProcessing => write!(f, "document_processing"),
            Self::FollowUp => write!(f, "follow_up"),
            Self::Validation => write!(f, "validation"),
            Self::Escalation => write!(f, "escalation"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_transitions() {
        assert!(WorkflowState::Pending.can_transition_to(WorkflowState::Active));
        assert!(WorkflowState::Active.can_transition_to(WorkflowState::Completed));
        assert!(!WorkflowState::Completed.can_transition_to(WorkflowState::Active));
    }
    
    #[test]
    fn test_task_transitions() {
        assert!(TaskState::Scheduled.can_transition_to(TaskState::Running));
        assert!(TaskState::Failed.can_transition_to(TaskState::Running)); // Retry
        assert!(!TaskState::Completed.can_transition_to(TaskState::Running));
    }
}
