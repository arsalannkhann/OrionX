use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceDocument {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub file_name: String,
    pub file_type: String,
    pub file_size: i64,
    pub upload_date: DateTime<Utc>,
    pub processed_date: Option<DateTime<Utc>>,
    pub extracted_data: Option<ExtractionResult>,
    pub processing_status: ProcessingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub cas_numbers: Vec<CASExtraction>,
    pub test_results: Vec<TestResultExtraction>,
    pub certifications: Vec<CertificationExtraction>,
    pub confidence: f64,
    pub uncertainties: Vec<Uncertainty>,
    pub processing_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CASExtraction {
    pub cas_number: String,
    pub confidence: f64,
    pub context: String,
    pub source_location: DocumentLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResultExtraction {
    pub test_type: String,
    pub value: f64,
    pub unit: String,
    pub confidence: f64,
    pub source_location: DocumentLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationExtraction {
    pub certification_type: String,
    pub issuing_body: String,
    pub certificate_number: String,
    pub confidence: f64,
    pub source_location: DocumentLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLocation {
    pub page: Option<u32>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Uncertainty {
    pub field: String,
    pub reason: String,
    pub confidence: f64,
    pub alternatives: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Uploaded,
    Processing,
    Processed,
    Failed,
    RequiresReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    TestReport,
    CertificateOfAnalysis,
    SafetyDataSheet,
    ComplianceDeclaration,
    MaterialSpecification,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DocumentReference {
    pub document_id: Uuid,
    pub page: Option<u32>,
    pub section: Option<String>,
    pub extraction_timestamp: DateTime<Utc>,
}

impl Default for ComplianceDocument {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            file_name: String::new(),
            file_type: String::new(),
            file_size: 0,
            upload_date: Utc::now(),
            processed_date: None,
            extracted_data: None,
            processing_status: ProcessingStatus::Uploaded,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}