//! Supplier domain models for the Elementa compliance system.
//! 
//! This module defines the core supplier-related data structures including
//! supplier records, contact information, compliance history, and risk profiles.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::{Validate, ValidationError};

/// Represents a supplier in the compliance system with full contact information,
/// compliance history, and risk assessment data.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate, PartialEq)]
pub struct SupplierRecord {
    pub id: Uuid,
    #[validate(length(min = 1, max = 255, message = "Supplier name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate]
    pub contact_info: ContactInfo,
    pub relationship: SupplierRelationship,
    pub compliance_history: Vec<ComplianceHistoryEntry>,
    #[validate]
    pub communication_preferences: CommunicationPreferences,
    #[validate]
    pub risk_profile: RiskProfile,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Contact information for a supplier including email addresses, phone, and physical address.
#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct ContactInfo {
    #[validate(email(message = "Primary email must be a valid email address"))]
    pub primary_email: String,
    #[validate(custom = "validate_email_list")]
    pub alternate_emails: Vec<String>,
    #[validate(length(min = 1, max = 255, message = "Contact person name must be between 1 and 255 characters"))]
    pub contact_person: String,
    pub phone: Option<String>,
    #[validate]
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct Address {
    #[validate(length(min = 1, max = 255, message = "Street address is required"))]
    pub street: String,
    #[validate(length(min = 1, max = 100, message = "City is required"))]
    pub city: String,
    #[validate(length(max = 100))]
    pub state: Option<String>,
    #[validate(length(min = 1, max = 20, message = "Postal code is required"))]
    pub postal_code: String,
    #[validate(length(min = 2, max = 3, message = "Country code must be 2-3 characters"))]
    pub country: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SupplierRelationship {
    Strategic,
    Preferred,
    Standard,
    NewVendor,
    AtRisk,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ComplianceHistoryEntry {
    pub campaign_id: Uuid,
    pub status: ComplianceStatus,
    pub response_time_days: Option<i32>,
    pub completeness_score: f64,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceStatus {
    NotStarted,
    InProgress,
    PartiallyComplete,
    Complete,
    NonCompliant,
    Escalated,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct CommunicationPreferences {
    #[validate(length(min = 2, max = 5, message = "Language code must be 2-5 characters"))]
    pub preferred_language: String,
    pub technical_level: TechnicalLevel,
    pub response_format: ResponseFormat,
    #[validate(range(min = 1, max = 30, message = "Follow-up frequency must be between 1 and 30 days"))]
    pub follow_up_frequency_days: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TechnicalLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResponseFormat {
    Email,
    Portal,
    Phone,
    Document,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, PartialEq)]
pub struct RiskProfile {
    pub compliance_risk: RiskLevel,
    pub response_reliability: RiskLevel,
    pub data_quality: RiskLevel,
    #[validate(range(min = 0.0, max = 1.0, message = "Overall score must be between 0.0 and 1.0"))]
    pub overall_score: f64,
    pub last_assessed: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for SupplierRecord {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            contact_info: ContactInfo::default(),
            relationship: SupplierRelationship::Standard,
            compliance_history: Vec::new(),
            communication_preferences: CommunicationPreferences::default(),
            risk_profile: RiskProfile::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

impl ContactInfo {
    /// Validates the phone number format if present
    pub fn validate_phone(&self) -> bool {
        if let Some(phone) = &self.phone {
            let phone_regex = regex::Regex::new(r"^\+?[\d\s\-\(\)]{7,20}$").unwrap();
            phone_regex.is_match(phone)
        } else {
            true // None is valid
        }
    }
    
    /// Sets the phone number after validation
    pub fn set_phone(&mut self, phone: Option<String>) -> Result<(), String> {
        if let Some(ref phone_str) = phone {
            let phone_regex = regex::Regex::new(r"^\+?[\d\s\-\(\)]{7,20}$").unwrap();
            if !phone_regex.is_match(phone_str) {
                return Err("Invalid phone number format".to_string());
            }
        }
        self.phone = phone;
        Ok(())
    }
}

impl Default for ContactInfo {
    fn default() -> Self {
        Self {
            primary_email: String::new(),
            alternate_emails: Vec::new(),
            contact_person: String::new(),
            phone: None,
            address: None,
        }
    }
}

impl Default for CommunicationPreferences {
    fn default() -> Self {
        Self {
            preferred_language: "en".to_string(),
            technical_level: TechnicalLevel::Intermediate,
            response_format: ResponseFormat::Email,
            follow_up_frequency_days: 7,
        }
    }
}

impl Default for RiskProfile {
    fn default() -> Self {
        Self {
            compliance_risk: RiskLevel::Medium,
            response_reliability: RiskLevel::Medium,
            data_quality: RiskLevel::Medium,
            overall_score: 0.5,
            last_assessed: Utc::now(),
        }
    }
}

// Custom validation functions
fn validate_email_list(emails: &[String]) -> Result<(), ValidationError> {
    for email in emails {
        if !validator::validate_email(email) {
            return Err(ValidationError::new("invalid_email"));
        }
    }
    Ok(())
}

// Utility methods for SupplierRecord
impl SupplierRecord {
    /// Creates a new supplier record with the given name and email
    pub fn new(name: String, primary_email: String, contact_person: String) -> Self {
        let mut record = Self::default();
        record.name = name;
        record.contact_info.primary_email = primary_email;
        record.contact_info.contact_person = contact_person;
        record
    }
    
    /// Updates the supplier's risk profile based on compliance history
    pub fn update_risk_profile(&mut self) {
        let history_count = self.compliance_history.len();
        if history_count == 0 {
            return;
        }
        
        let compliant_count = self.compliance_history.iter()
            .filter(|h| matches!(h.status, ComplianceStatus::Complete))
            .count();
        
        let compliance_rate = compliant_count as f64 / history_count as f64;
        
        // Update compliance risk based on compliance rate
        self.risk_profile.compliance_risk = match compliance_rate {
            r if r >= 0.9 => RiskLevel::Low,
            r if r >= 0.7 => RiskLevel::Medium,
            r if r >= 0.5 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
        
        // Calculate average response time
        let avg_response_time = self.compliance_history.iter()
            .filter_map(|h| h.response_time_days)
            .sum::<i32>() as f64 / history_count as f64;
        
        // Update response reliability based on response time
        self.risk_profile.response_reliability = match avg_response_time {
            t if t <= 3.0 => RiskLevel::Low,
            t if t <= 7.0 => RiskLevel::Medium,
            t if t <= 14.0 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
        
        // Calculate overall score
        let compliance_score = match self.risk_profile.compliance_risk {
            RiskLevel::Low => 0.9,
            RiskLevel::Medium => 0.7,
            RiskLevel::High => 0.4,
            RiskLevel::Critical => 0.1,
        };
        
        let reliability_score = match self.risk_profile.response_reliability {
            RiskLevel::Low => 0.9,
            RiskLevel::Medium => 0.7,
            RiskLevel::High => 0.4,
            RiskLevel::Critical => 0.1,
        };
        
        self.risk_profile.overall_score = (compliance_score + reliability_score) / 2.0;
        self.risk_profile.last_assessed = Utc::now();
        self.updated_at = Utc::now();
    }
    
    /// Checks if the supplier is high risk
    pub fn is_high_risk(&self) -> bool {
        matches!(self.risk_profile.compliance_risk, RiskLevel::High | RiskLevel::Critical) ||
        matches!(self.risk_profile.response_reliability, RiskLevel::High | RiskLevel::Critical) ||
        self.risk_profile.overall_score < 0.5
    }
    
    /// Gets the primary contact email
    pub fn primary_email(&self) -> &str {
        &self.contact_info.primary_email
    }
    
    /// Gets all contact emails (primary + alternates)
    pub fn all_emails(&self) -> Vec<&str> {
        let mut emails = vec![self.contact_info.primary_email.as_str()];
        emails.extend(self.contact_info.alternate_emails.iter().map(|s| s.as_str()));
        emails
    }
    
    /// Adds a compliance history entry
    pub fn add_compliance_history(&mut self, entry: ComplianceHistoryEntry) {
        self.compliance_history.push(entry);
        self.update_risk_profile();
    }
}