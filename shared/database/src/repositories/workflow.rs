//! Workflow Repository
//!
//! CRUD operations for workflow instances.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::{WorkflowInstance, WorkflowStatus};

pub struct WorkflowRepository {
    pool: PgPool,
}

impl WorkflowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find workflow by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<WorkflowInstance>> {
        let row: Option<WorkflowRow> = sqlx::query_as(
            r#"
            SELECT id, client_id, campaign_name, status, suppliers,
                   start_date, deadline, progress, escalations,
                   created_at, updated_at
            FROM workflows
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch workflow by ID")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find workflows by status
    pub async fn find_by_status(&self, status: WorkflowStatus) -> Result<Vec<WorkflowInstance>> {
        let status_str = serde_json::to_string(&status)?.trim_matches('"').to_string();
        
        let rows: Vec<WorkflowRow> = sqlx::query_as(
            r#"
            SELECT id, client_id, campaign_name, status, suppliers,
                   start_date, deadline, progress, escalations,
                   created_at, updated_at
            FROM workflows
            WHERE status = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch workflows by status")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find active workflows
    pub async fn find_active(&self) -> Result<Vec<WorkflowInstance>> {
        let rows: Vec<WorkflowRow> = sqlx::query_as(
            r#"
            SELECT id, client_id, campaign_name, status, suppliers,
                   start_date, deadline, progress, escalations,
                   created_at, updated_at
            FROM workflows
            WHERE status IN ('InProgress', 'Created')
            ORDER BY deadline ASC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch active workflows")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Create new workflow
    pub async fn create(&self, workflow: WorkflowInstance) -> Result<WorkflowInstance> {
        let suppliers = serde_json::to_value(&workflow.suppliers)?;
        let status_str = serde_json::to_string(&workflow.status)?.trim_matches('"').to_string();
        let progress = serde_json::to_value(&workflow.progress)?;
        let escalations = serde_json::to_value(&workflow.escalations)?;
        let now = Utc::now();
        
        let row: WorkflowRow = sqlx::query_as(
            r#"
            INSERT INTO workflows 
                (id, client_id, campaign_name, status, suppliers,
                 start_date, deadline, progress, escalations,
                 created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, client_id, campaign_name, status, suppliers,
                      start_date, deadline, progress, escalations,
                      created_at, updated_at
            "#
        )
        .bind(workflow.id)
        .bind(workflow.client_id)
        .bind(&workflow.campaign_name)
        .bind(&status_str)
        .bind(&suppliers)
        .bind(workflow.start_date)
        .bind(workflow.deadline)
        .bind(&progress)
        .bind(&escalations)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create workflow")?;
        
        Ok(row.into())
    }
    
    /// Update workflow status
    pub async fn update_status(&self, id: Uuid, status: WorkflowStatus) -> Result<bool> {
        let status_str = serde_json::to_string(&status)?.trim_matches('"').to_string();
        
        let result = sqlx::query(
            "UPDATE workflows SET status = $2, updated_at = $3 WHERE id = $1"
        )
        .bind(id)
        .bind(&status_str)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .context("Failed to update workflow status")?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Delete workflow
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM workflows WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete workflow")?;
        
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, FromRow)]
struct WorkflowRow {
    id: Uuid,
    client_id: Uuid,
    campaign_name: String,
    status: String,
    suppliers: serde_json::Value,
    start_date: chrono::DateTime<Utc>,
    deadline: chrono::DateTime<Utc>,
    progress: serde_json::Value,
    escalations: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<WorkflowRow> for WorkflowInstance {
    fn from(row: WorkflowRow) -> Self {
        Self {
            id: row.id,
            client_id: row.client_id,
            campaign_name: row.campaign_name,
            status: serde_json::from_str(&format!("\"{}\"", row.status))
                .unwrap_or(WorkflowStatus::Created),
            suppliers: serde_json::from_value(row.suppliers).unwrap_or_default(),
            start_date: row.start_date,
            deadline: row.deadline,
            progress: serde_json::from_value(row.progress).unwrap_or_default(),
            escalations: serde_json::from_value(row.escalations).unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
