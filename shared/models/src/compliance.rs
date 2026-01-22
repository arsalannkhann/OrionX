//! Compliance domain models for the Elementa compliance system.
//! 
//! This module defines compliance-related data structures including
//! compliance records, CAS records, test results, and certifications.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

use crate::{AuditEntry, DocumentReference};

/// Represents a compliance record containing all compliance data for a specific
/// supplier-component pair, including CAS records, test results, and certifications.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate, PartialEq)]
pub struct ComplianceRecord {
    pub id: Uuid,
    pub supplier_id: Uuid,
    pub component_id: Uuid,
    #[validate(custom = "validate_cas_records")]
    pub cas_records: Vec<CASRecord>,
    pub test_results: Vec<TestResult>,
    pub certifications: Vec<Certification>,
    pub submission_date: DateTime<Utc>,
    pub validation_status: ValidationStatus,
    pub audit_trail: Vec<AuditEntry>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Represents a CAS (Chemical Abstracts Service) record with chemical identification,
/// PFAS classification, and regulatory status information.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct CASRecord {
    #[validate(custom = "validate_cas_number")]
    pub cas_number: String,
    #[validate(length(min = 1, max = 255, message = "Chemical name is required"))]
    pub chemical_name: String,
    pub is_pfas: bool,
    #[validate(range(min = 0.0, max = 1.0, message = "Confidence must be between 0.0 and 1.0"))]
    pub confidence: f64,
    #[validate]
    pub regulatory_status: RegulatoryStatus,
    pub source_document: DocumentReference,
    pub extraction_method: ExtractionMethod,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExtractionMethod {
    VLMAutomatic,
    OCRProcessing,
    ManualEntry,
    DatabaseLookup,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct RegulatoryStatus {
    pub regulatory_lists: Vec<RegulatoryList>,
    pub reporting_requirements: Vec<ReportingRequirement>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct RegulatoryList {
    #[validate(length(min = 1, max = 100, message = "Source is required"))]
    pub source: String, // EPA, OECD, etc.
    #[validate(length(min = 1, max = 200, message = "List name is required"))]
    pub list_name: String,
    pub date_added: DateTime<Utc>,
    #[validate(range(min = 0.0, message = "Reporting threshold must be positive"))]
    pub reporting_threshold: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ReportingRequirement {
    #[validate(length(min = 1, max = 100, message = "Regulation name is required"))]
    pub regulation: String,
    pub deadline: DateTime<Utc>,
    #[validate(range(min = 0.0, message = "Threshold must be positive"))]
    pub threshold: Option<f64>,
    #[validate(length(min = 1, max = 100, message = "Reporting format is required"))]
    pub reporting_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct TestResult {
    pub test_type: TestType,
    #[validate(range(min = 0.0, message = "Result value must be positive"))]
    pub result_value: f64,
    #[validate(length(min = 1, max = 20, message = "Unit is required"))]
    pub unit: String,
    #[validate(range(min = 0.0, message = "Detection limit must be positive"))]
    pub detection_limit: Option<f64>,
    #[validate(length(min = 1, max = 100, message = "Test method is required"))]
    pub test_method: String,
    pub test_date: DateTime<Utc>,
    #[validate(length(min = 1, max = 200, message = "Laboratory name is required"))]
    pub laboratory: String,
    #[validate(length(max = 100))]
    pub certificate_number: Option<String>,
    pub source_document: DocumentReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TestType {
    PFASConcentration,
    ChemicalComposition,
    MaterialSafety,
    EnvironmentalImpact,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct Certification {
    pub certification_type: CertificationType,
    #[validate(length(min = 1, max = 200, message = "Issuing body is required"))]
    pub issuing_body: String,
    #[validate(length(min = 1, max = 100, message = "Certificate number is required"))]
    pub certificate_number: String,
    pub issue_date: DateTime<Utc>,
    pub expiry_date: Option<DateTime<Utc>>,
    #[validate(length(min = 1, max = 500, message = "Scope is required"))]
    pub scope: String,
    pub source_document: DocumentReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CertificationType {
    ISO14001,
    REACH,
    RoHS,
    PfasFree,
    MaterialSafety,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid,
    RequiresReview,
    Incomplete,
}

impl Default for ComplianceRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            supplier_id: Uuid::new_v4(),
            component_id: Uuid::new_v4(),
            cas_records: Vec::new(),
            test_results: Vec::new(),
            certifications: Vec::new(),
            submission_date: Utc::now(),
            validation_status: ValidationStatus::Pending,
            audit_trail: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// Custom validation functions
fn validate_cas_records(cas_records: &[CASRecord]) -> Result<(), ValidationError> {
    for record in cas_records {
        if let Err(e) = validate_cas_number(&record.cas_number) {
            return Err(e);
        }
    }
    Ok(())
}

fn validate_cas_number(cas_number: &str) -> Result<(), ValidationError> {
    // CAS number format: XXXXXX-XX-X where X is a digit
    let parts: Vec<&str> = cas_number.split('-').collect();
    if parts.len() != 3 {
        return Err(ValidationError::new("invalid_cas_format"));
    }
    
    // Check format: 2-7 digits, 2 digits, 1 digit
    if parts[0].len() < 2 || parts[0].len() > 7 || parts[1].len() != 2 || parts[2].len() != 1 {
        return Err(ValidationError::new("invalid_cas_format"));
    }
    
    // Check all parts are numeric
    if !parts.iter().all(|part| part.chars().all(|c| c.is_ascii_digit())) {
        return Err(ValidationError::new("invalid_cas_format"));
    }
    
    Ok(())
}

// Utility methods for ComplianceRecord
impl ComplianceRecord {
    /// Creates a new compliance record for a supplier and component
    pub fn new(supplier_id: Uuid, component_id: Uuid) -> Self {
        let mut record = Self::default();
        record.supplier_id = supplier_id;
        record.component_id = component_id;
        record
    }
    
    /// Adds a CAS record to the compliance record
    pub fn add_cas_record(&mut self, cas_record: CASRecord) {
        self.cas_records.push(cas_record);
        self.updated_at = Utc::now();
        self.update_validation_status();
    }
    
    /// Adds a test result to the compliance record
    pub fn add_test_result(&mut self, test_result: TestResult) {
        self.test_results.push(test_result);
        self.updated_at = Utc::now();
        self.update_validation_status();
    }
    
    /// Adds a certification to the compliance record
    pub fn add_certification(&mut self, certification: Certification) {
        self.certifications.push(certification);
        self.updated_at = Utc::now();
        self.update_validation_status();
    }
    
    /// Updates the validation status based on available data
    pub fn update_validation_status(&mut self) {
        if self.cas_records.is_empty() && self.test_results.is_empty() && self.certifications.is_empty() {
            self.validation_status = ValidationStatus::Incomplete;
        } else if self.has_low_confidence_data() {
            self.validation_status = ValidationStatus::RequiresReview;
        } else if self.has_complete_data() {
            self.validation_status = ValidationStatus::Valid;
        } else {
            self.validation_status = ValidationStatus::Incomplete;
        }
    }
    
    /// Checks if the record has low confidence data that requires review
    pub fn has_low_confidence_data(&self) -> bool {
        self.cas_records.iter().any(|r| r.confidence < 0.7)
    }
    
    /// Checks if the record has complete compliance data
    pub fn has_complete_data(&self) -> bool {
        !self.cas_records.is_empty() && 
        self.cas_records.iter().all(|r| r.confidence >= 0.7)
    }
    
    /// Gets all PFAS substances in this record
    pub fn pfas_substances(&self) -> Vec<&CASRecord> {
        self.cas_records.iter().filter(|r| r.is_pfas).collect()
    }
    
    /// Gets all non-PFAS substances in this record
    pub fn non_pfas_substances(&self) -> Vec<&CASRecord> {
        self.cas_records.iter().filter(|r| !r.is_pfas).collect()
    }
    
    /// Checks if the record contains any PFAS substances
    pub fn contains_pfas(&self) -> bool {
        self.cas_records.iter().any(|r| r.is_pfas)
    }
    
    /// Gets the overall confidence score for the record
    pub fn overall_confidence(&self) -> f64 {
        if self.cas_records.is_empty() {
            0.0
        } else {
            self.cas_records.iter().map(|r| r.confidence).sum::<f64>() / self.cas_records.len() as f64
        }
    }
}

// Utility methods for CASRecord
impl CASRecord {
    /// Creates a new CAS record
    pub fn new(
        cas_number: String,
        chemical_name: String,
        is_pfas: bool,
        confidence: f64,
        source_document: DocumentReference,
        extraction_method: ExtractionMethod,
    ) -> Self {
        Self {
            cas_number,
            chemical_name,
            is_pfas,
            confidence,
            regulatory_status: RegulatoryStatus {
                regulatory_lists: Vec::new(),
                reporting_requirements: Vec::new(),
                last_updated: Utc::now(),
            },
            source_document,
            extraction_method,
            created_at: Utc::now(),
        }
    }
    
    /// Checks if this CAS record requires regulatory reporting
    pub fn requires_reporting(&self) -> bool {
        !self.regulatory_status.reporting_requirements.is_empty()
    }
    
    /// Gets upcoming reporting deadlines
    pub fn upcoming_deadlines(&self) -> Vec<&ReportingRequirement> {
        let now = Utc::now();
        self.regulatory_status.reporting_requirements
            .iter()
            .filter(|req| req.deadline > now)
            .collect()
    }
}