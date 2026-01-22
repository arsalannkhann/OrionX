//! Workflow Scheduler
//! 
//! Handles task scheduling, follow-up timing, and deadline management.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::state_machine::TaskType;
use crate::WorkflowConfig;

/// Scheduled task
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub supplier_id: Uuid,
    pub task_type: TaskType,
    pub scheduled_at: DateTime<Utc>,
    pub priority: i32,
}

/// Scheduler for workflow tasks
#[allow(dead_code)]
pub struct WorkflowScheduler {
    config: WorkflowConfig,
}

#[allow(dead_code)]
impl WorkflowScheduler {
    pub fn new(config: WorkflowConfig) -> Self {
        Self { config }
    }
    
    /// Schedule initial outreach tasks for all suppliers
    pub fn schedule_initial_outreach(&self, workflow_id: Uuid, supplier_ids: &[Uuid]) -> Vec<ScheduledTask> {
        let now = Utc::now();
        
        supplier_ids.iter().enumerate().map(|(i, &supplier_id)| {
            // Stagger outreach to avoid overwhelming email servers
            let delay_minutes = (i as i64) * 2; // 2 minutes between each
            
            ScheduledTask {
                id: Uuid::new_v4(),
                workflow_id,
                supplier_id,
                task_type: TaskType::InitialOutreach,
                scheduled_at: now + Duration::minutes(delay_minutes),
                priority: 100, // High priority for initial outreach
            }
        }).collect()
    }
    
    /// Schedule follow-up task
    pub fn schedule_follow_up(&self, workflow_id: Uuid, supplier_id: Uuid, follow_up_number: i32) -> Option<ScheduledTask> {
        if follow_up_number >= self.config.max_follow_ups {
            return None; // No more follow-ups allowed
        }
        
        let delay_days = self.config.follow_up_interval_days * (follow_up_number + 1);
        let scheduled_at = Utc::now() + Duration::days(delay_days as i64);
        
        Some(ScheduledTask {
            id: Uuid::new_v4(),
            workflow_id,
            supplier_id,
            task_type: TaskType::FollowUp,
            scheduled_at,
            priority: 80 - (follow_up_number * 10), // Lower priority for later follow-ups
        })
    }
    
    /// Schedule document processing task
    pub fn schedule_document_processing(&self, workflow_id: Uuid, supplier_id: Uuid) -> ScheduledTask {
        ScheduledTask {
            id: Uuid::new_v4(),
            workflow_id,
            supplier_id,
            task_type: TaskType::DocumentProcessing,
            scheduled_at: Utc::now(), // Immediate processing
            priority: 90,
        }
    }
    
    /// Schedule validation task
    pub fn schedule_validation(&self, workflow_id: Uuid, supplier_id: Uuid) -> ScheduledTask {
        ScheduledTask {
            id: Uuid::new_v4(),
            workflow_id,
            supplier_id,
            task_type: TaskType::Validation,
            scheduled_at: Utc::now() + Duration::minutes(5), // Small delay after processing
            priority: 85,
        }
    }
    
    /// Check if escalation is needed
    pub fn should_escalate(&self, last_contact: DateTime<Utc>, follow_up_count: i32) -> bool {
        if !self.config.auto_escalate {
            return false;
        }
        
        let days_since_contact = (Utc::now() - last_contact).num_days();
        
        // Escalate if past threshold and max follow-ups exhausted
        days_since_contact >= self.config.escalation_threshold_days as i64 
            && follow_up_count >= self.config.max_follow_ups
    }
    
    /// Schedule escalation task
    pub fn schedule_escalation(&self, workflow_id: Uuid, supplier_id: Uuid) -> ScheduledTask {
        ScheduledTask {
            id: Uuid::new_v4(),
            workflow_id,
            supplier_id,
            task_type: TaskType::Escalation,
            scheduled_at: Utc::now(), // Immediate escalation
            priority: 100, // Highest priority
        }
    }
    
    /// Calculate deadline risk
    pub fn calculate_deadline_risk(&self, deadline: DateTime<Utc>, progress_percent: f64) -> DeadlineRisk {
        let days_remaining = (deadline - Utc::now()).num_days();
        
        // Expected progress based on time
        let total_duration_days = 30.0; // Assume 30 day campaigns
        let expected_progress = (1.0 - (days_remaining as f64 / total_duration_days)) * 100.0;
        
        let progress_gap = expected_progress - progress_percent;
        
        if days_remaining <= 0 {
            DeadlineRisk::Critical
        } else if days_remaining <= 7 && progress_percent < 80.0 {
            DeadlineRisk::High
        } else if progress_gap > 20.0 {
            DeadlineRisk::Medium
        } else {
            DeadlineRisk::Low
        }
    }
}

/// Deadline risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DeadlineRisk {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for DeadlineRisk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

impl Default for WorkflowScheduler {
    fn default() -> Self {
        Self::new(WorkflowConfig::default())
    }
}
