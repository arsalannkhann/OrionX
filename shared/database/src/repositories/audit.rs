//! Audit Repository
//!
//! Immutable audit trail with hash chain verification.

use anyhow::{Context, Result};
use chrono::Utc;
use sha2::{Sha256, Digest};
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::AuditEntry;

pub struct AuditRepository {
    pool: PgPool,
}

impl AuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Create new audit entry (immutable - no update/delete)
    pub async fn create(&self, entry: AuditEntry, previous_hash: Option<String>) -> Result<AuditEntry> {
        let action = serde_json::to_string(&entry.action)?;
        let details = serde_json::to_value(&entry.details)?;
        let source_document = serde_json::to_value(&entry.source_document)?;
        
        // Calculate hash including previous hash for chain integrity
        let hash = self.calculate_hash(&entry, previous_hash.as_deref());
        
        let row: AuditRow = sqlx::query_as(
            r#"
            INSERT INTO audit_entries 
                (id, timestamp, action, user_id, agent_id, details,
                 source_document, hash, previous_hash, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, timestamp, action, user_id, agent_id, details,
                      source_document, hash, previous_hash, created_at
            "#
        )
        .bind(entry.id)
        .bind(entry.timestamp)
        .bind(action.trim_matches('"'))
        .bind(entry.user_id)
        .bind(&entry.agent_id)
        .bind(&details)
        .bind(&source_document)
        .bind(&hash)
        .bind(&previous_hash)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to create audit entry")?;
        
        Ok(row.into())
    }
    
    /// Find audit entries for an entity
    pub async fn find_by_entity(&self, entity_type: &str, entity_id: Uuid) -> Result<Vec<AuditEntry>> {
        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, timestamp, action, user_id, agent_id, details,
                   source_document, hash, previous_hash, created_at
            FROM audit_entries
            WHERE details->>'entity_type' = $1 AND (details->>'entity_id')::uuid = $2
            ORDER BY timestamp ASC
            "#
        )
        .bind(entity_type)
        .bind(entity_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit entries by entity")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Verify hash chain integrity for a date range
    pub async fn verify_chain(&self, from: chrono::DateTime<Utc>, to: chrono::DateTime<Utc>) -> Result<ChainVerification> {
        let rows: Vec<AuditRow> = sqlx::query_as(
            r#"
            SELECT id, timestamp, action, user_id, agent_id, details,
                   source_document, hash, previous_hash, created_at
            FROM audit_entries
            WHERE timestamp >= $1 AND timestamp <= $2
            ORDER BY timestamp ASC
            "#
        )
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch audit entries for verification")?;
        
        let mut broken_links = Vec::new();
        let mut previous_hash: Option<String> = None;
        
        for row in &rows {
            let entry: AuditEntry = row.clone().into();
            let expected_hash = self.calculate_hash(&entry, previous_hash.as_deref());
            
            if row.hash != expected_hash {
                broken_links.push(row.id);
            }
            
            previous_hash = Some(row.hash.clone());
        }
        
        Ok(ChainVerification {
            is_valid: broken_links.is_empty(),
            entries_verified: rows.len(),
            broken_links,
        })
    }
    
    fn calculate_hash(&self, entry: &AuditEntry, previous_hash: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update(entry.id.to_string().as_bytes());
        hasher.update(entry.timestamp.to_rfc3339().as_bytes());
        hasher.update(format!("{:?}", entry.action).as_bytes());
        hasher.update(serde_json::to_string(&entry.details).unwrap_or_default().as_bytes());
        
        if let Some(prev) = previous_hash {
            hasher.update(prev.as_bytes());
        }
        
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, Clone, FromRow)]
struct AuditRow {
    id: Uuid,
    timestamp: chrono::DateTime<Utc>,
    action: String,
    user_id: Option<Uuid>,
    agent_id: Option<String>,
    details: serde_json::Value,
    source_document: serde_json::Value,
    hash: String,
    previous_hash: Option<String>,
    created_at: chrono::DateTime<Utc>,
}

impl From<AuditRow> for AuditEntry {
    fn from(row: AuditRow) -> Self {
        use elementa_models::{AuditAction, AuditDetails};
        
        Self {
            id: row.id,
            timestamp: row.timestamp,
            action: serde_json::from_str(&format!("\"{}\"", row.action))
                .unwrap_or(AuditAction::SystemAction),
            user_id: row.user_id,
            agent_id: row.agent_id,
            details: serde_json::from_value(row.details).unwrap_or_else(|_| AuditDetails {
                entity_type: String::new(),
                entity_id: uuid::Uuid::nil(),
                changes: Vec::new(),
                metadata: std::collections::HashMap::new(),
            }),
            source_document: serde_json::from_value(row.source_document).ok(),
            hash: row.hash,
            previous_hash: row.previous_hash,
            created_at: row.created_at,
        }
    }
}

#[derive(Debug)]
pub struct ChainVerification {
    pub is_valid: bool,
    pub entries_verified: usize,
    pub broken_links: Vec<Uuid>,
}
