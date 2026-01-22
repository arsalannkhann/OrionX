//! Email Repository
//!
//! CRUD operations for email communications.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::{EmailCommunication, EmailDirection, DeliveryStatus, EmailProcessingStatus};

pub struct EmailRepository {
    pool: PgPool,
}

impl EmailRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find email by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<EmailCommunication>> {
        let row: Option<EmailRow> = sqlx::query_as(
            r#"
            SELECT id, thread_id, supplier_id, direction, subject, body,
                   sent_at, received_at, attachments, delivery_status,
                   processing_status, created_at, updated_at
            FROM email_communications
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch email by ID")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find emails in a thread
    pub async fn find_by_thread(&self, thread_id: &str) -> Result<Vec<EmailCommunication>> {
        let rows: Vec<EmailRow> = sqlx::query_as(
            r#"
            SELECT id, thread_id, supplier_id, direction, subject, body,
                   sent_at, received_at, attachments, delivery_status,
                   processing_status, created_at, updated_at
            FROM email_communications
            WHERE thread_id = $1
            ORDER BY created_at ASC
            "#
        )
        .bind(thread_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch emails by thread")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find emails for a supplier
    pub async fn find_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<EmailCommunication>> {
        let rows: Vec<EmailRow> = sqlx::query_as(
            r#"
            SELECT id, thread_id, supplier_id, direction, subject, body,
                   sent_at, received_at, attachments, delivery_status,
                   processing_status, created_at, updated_at
            FROM email_communications
            WHERE supplier_id = $1
            ORDER BY created_at DESC
            "#
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch emails by supplier")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Create new email
    pub async fn create(&self, email: EmailCommunication) -> Result<EmailCommunication> {
        let attachments = serde_json::to_value(&email.attachments)?;
        let direction_str = serde_json::to_string(&email.direction)?.trim_matches('"').to_string();
        let delivery_str = serde_json::to_string(&email.delivery_status)?.trim_matches('"').to_string();
        let proc_str = serde_json::to_string(&email.processing_status)?.trim_matches('"').to_string();
        let now = Utc::now();
        
        let row: EmailRow = sqlx::query_as(
            r#"
            INSERT INTO email_communications 
                (id, thread_id, supplier_id, direction, subject, body,
                 sent_at, received_at, attachments, delivery_status,
                 processing_status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, thread_id, supplier_id, direction, subject, body,
                      sent_at, received_at, attachments, delivery_status,
                      processing_status, created_at, updated_at
            "#
        )
        .bind(email.id)
        .bind(&email.thread_id)
        .bind(email.supplier_id)
        .bind(&direction_str)
        .bind(&email.subject)
        .bind(&email.body)
        .bind(email.sent_at)
        .bind(email.received_at)
        .bind(&attachments)
        .bind(&delivery_str)
        .bind(&proc_str)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create email")?;
        
        Ok(row.into())
    }
    
    /// Update delivery status
    pub async fn update_delivery_status(&self, id: Uuid, status: DeliveryStatus) -> Result<bool> {
        let status_str = serde_json::to_string(&status)?.trim_matches('"').to_string();
        
        let result = sqlx::query(
            "UPDATE email_communications SET delivery_status = $2, updated_at = $3 WHERE id = $1"
        )
        .bind(id)
        .bind(&status_str)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .context("Failed to update delivery status")?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Update processing status
    pub async fn update_processing_status(&self, id: Uuid, status: EmailProcessingStatus) -> Result<bool> {
        let status_str = serde_json::to_string(&status)?.trim_matches('"').to_string();
        
        let result = sqlx::query(
            "UPDATE email_communications SET processing_status = $2, updated_at = $3 WHERE id = $1"
        )
        .bind(id)
        .bind(&status_str)
        .bind(Utc::now())
        .execute(&self.pool)
        .await
        .context("Failed to update processing status")?;
        
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, FromRow)]
struct EmailRow {
    id: Uuid,
    thread_id: String,
    supplier_id: Uuid,
    direction: String,
    subject: String,
    body: String,
    sent_at: Option<chrono::DateTime<Utc>>,
    received_at: Option<chrono::DateTime<Utc>>,
    attachments: serde_json::Value,
    delivery_status: String,
    processing_status: String,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<EmailRow> for EmailCommunication {
    fn from(row: EmailRow) -> Self {
        Self {
            id: row.id,
            thread_id: row.thread_id,
            supplier_id: row.supplier_id,
            direction: serde_json::from_str(&format!("\"{}\"", row.direction))
                .unwrap_or(EmailDirection::Outbound),
            subject: row.subject,
            body: row.body,
            sent_at: row.sent_at,
            received_at: row.received_at,
            attachments: serde_json::from_value(row.attachments).unwrap_or_default(),
            delivery_status: serde_json::from_str(&format!("\"{}\"", row.delivery_status))
                .unwrap_or(DeliveryStatus::Pending),
            processing_status: serde_json::from_str(&format!("\"{}\"", row.processing_status))
                .unwrap_or(EmailProcessingStatus::NotProcessed),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
