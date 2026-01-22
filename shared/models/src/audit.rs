use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::DocumentReference;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct AuditEntry {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: AuditAction,
    pub user_id: Option<Uuid>,
    pub agent_id: Option<String>,
    pub details: AuditDetails,
    pub source_document: Option<DocumentReference>,
    pub hash: String,
    pub previous_hash: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditAction {
    DocumentUploaded,
    DocumentProcessed,
    DataExtracted,
    ComplianceRecordCreated,
    ComplianceRecordUpdated,
    EmailSent,
    EmailReceived,
    WorkflowStarted,
    WorkflowCompleted,
    EscalationCreated,
    UserAction,
    SystemAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuditDetails {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub changes: Vec<FieldChange>,
    pub metadata: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldChange {
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChangeType {
    Created,
    Updated,
    Deleted,
    Accessed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOfCustody {
    pub document_id: Uuid,
    pub audit_entries: Vec<AuditEntry>,
    pub integrity_verified: bool,
    pub last_verification: DateTime<Utc>,
}

impl AuditEntry {
    pub fn new(
        action: AuditAction,
        entity_type: String,
        entity_id: Uuid,
        user_id: Option<Uuid>,
        agent_id: Option<String>,
    ) -> Self {
        let timestamp = Utc::now();
        let details = AuditDetails {
            entity_type,
            entity_id,
            changes: Vec::new(),
            metadata: std::collections::HashMap::new(),
        };
        
        let hash = Self::calculate_hash(&action, &details, &timestamp);
        
        Self {
            id: Uuid::new_v4(),
            timestamp,
            action,
            user_id,
            agent_id,
            details,
            source_document: None,
            hash,
            previous_hash: None,
            created_at: timestamp,
        }
    }
    
    fn calculate_hash(action: &AuditAction, details: &AuditDetails, timestamp: &DateTime<Utc>) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_string(action).unwrap_or_default());
        hasher.update(serde_json::to_string(details).unwrap_or_default());
        hasher.update(timestamp.to_rfc3339());
        
        hex::encode(hasher.finalize())
    }
    
    pub fn verify_integrity(&self) -> bool {
        let calculated_hash = Self::calculate_hash(&self.action, &self.details, &self.timestamp);
        calculated_hash == self.hash
    }
}