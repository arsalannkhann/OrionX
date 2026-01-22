use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use crate::{SupplierRecord, Component, TechnicalLevel};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailCommunication {
    pub id: Uuid,
    pub thread_id: String,
    pub supplier_id: Uuid,
    pub direction: EmailDirection,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<EmailAttachment>,
    pub sent_at: Option<DateTime<Utc>>,
    pub received_at: Option<DateTime<Utc>>,
    pub delivery_status: DeliveryStatus,
    pub processing_status: EmailProcessingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailDirection {
    Outbound,
    Inbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAttachment {
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub document_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Bounced,
    Failed,
    SpamFiltered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailProcessingStatus {
    NotProcessed,
    Processing,
    Processed,
    RequiresReview,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequest {
    pub supplier: SupplierRecord,
    pub components: Vec<Component>,
    pub deadline: DateTime<Utc>,
    pub personalization: PersonalizationContext,
    pub attachments: Vec<EmailAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalizationContext {
    pub supplier_relationship: String,
    pub previous_interactions: u32,
    pub industry_context: Option<String>,
    pub regulatory_focus: Vec<String>,
    pub custom_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContent {
    pub subject: String,
    pub body: String,
    pub tone: CommunicationTone,
    pub technical_level: TechnicalLevel,
    pub template_used: String,
    pub personalization_applied: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationTone {
    Formal,
    Professional,
    Friendly,
    Urgent,
    Collaborative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEmail {
    pub message_id: String,
    pub thread_id: String,
    pub from_address: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<EmailAttachment>,
    pub received_at: DateTime<Utc>,
    pub supplier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub classification: EmailClassification,
    pub extracted_data: Option<String>,
    pub requires_response: bool,
    pub suggested_response: Option<String>,
    pub escalation_required: bool,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailClassification {
    ComplianceResponse,
    Question,
    Clarification,
    Objection,
    OutOfOffice,
    Spam,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUpContext {
    pub supplier_id: Uuid,
    pub previous_emails: Vec<Uuid>,
    pub missing_information: Vec<String>,
    pub attempt_number: u32,
    pub escalation_level: u32,
    pub deadline: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    pub thread_id: String,
    pub supplier_id: Uuid,
    pub messages: Vec<Uuid>,
    pub current_status: ConversationStatus,
    pub last_activity: DateTime<Utc>,
    pub context_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationStatus {
    AwaitingResponse,
    ResponseReceived,
    RequiresFollowUp,
    Escalated,
    Completed,
    Stalled,
}

impl Default for EmailCommunication {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            thread_id: String::new(),
            supplier_id: Uuid::new_v4(),
            direction: EmailDirection::Outbound,
            subject: String::new(),
            body: String::new(),
            attachments: Vec::new(),
            sent_at: None,
            received_at: None,
            delivery_status: DeliveryStatus::Pending,
            processing_status: EmailProcessingStatus::NotProcessed,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}