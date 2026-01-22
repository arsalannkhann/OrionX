//! Chemical substance domain models for the Elementa compliance system.
//! 
//! This module defines chemical-related data structures including
//! chemical substances, PFAS classifications, and regulatory information.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::{Validate, ValidationError};

/// Represents a chemical substance with CAS number, PFAS classification,
/// and comprehensive regulatory status information.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow, Validate)]
pub struct ChemicalSubstance {
    #[validate(custom = "validate_cas_number")]
    pub cas_number: String,
    #[validate(length(min = 1, max = 255, message = "Chemical name is required"))]
    pub chemical_name: String,
    #[validate(length(max = 50))]
    pub molecular_formula: Option<String>,
    #[validate(range(min = 0.0, message = "Molecular weight must be positive"))]
    pub molecular_weight: Option<f64>,
    pub is_pfas: bool,
    #[validate]
    pub pfas_classification: Option<PFASClassification>,
    #[validate]
    pub regulatory_status: RegulatoryStatus,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PFASClassification {
    pub is_pfas: bool,
    #[validate(range(min = 0.0, max = 1.0, message = "Confidence must be between 0.0 and 1.0"))]
    pub confidence: f64,
    pub regulatory_lists: Vec<RegulatoryList>,
    pub reporting_requirements: Vec<ReportingRequirement>,
    #[validate(length(min = 1, max = 100, message = "Classification source is required"))]
    pub classification_source: String,
    pub classification_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegulatoryList {
    #[validate(length(min = 1, max = 100, message = "Source is required"))]
    pub source: String, // EPA, OECD, etc.
    #[validate(length(min = 1, max = 200, message = "List name is required"))]
    pub list_name: String,
    pub date_added: DateTime<Utc>,
    #[validate(range(min = 0.0, message = "Reporting threshold must be positive"))]
    pub reporting_threshold: Option<f64>,
    #[validate(length(min = 1, max = 50, message = "List version is required"))]
    pub list_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ReportingRequirement {
    #[validate(length(min = 1, max = 100, message = "Regulation name is required"))]
    pub regulation: String,
    #[validate(length(min = 1, max = 100, message = "Jurisdiction is required"))]
    pub jurisdiction: String,
    pub deadline: DateTime<Utc>,
    #[validate(range(min = 0.0, message = "Threshold must be positive"))]
    pub threshold: Option<f64>,
    #[validate(length(min = 1, max = 100, message = "Reporting format is required"))]
    pub reporting_format: String,
    pub mandatory: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RegulatoryStatus {
    pub regulatory_lists: Vec<RegulatoryList>,
    pub reporting_requirements: Vec<ReportingRequirement>,
    pub restrictions: Vec<ChemicalRestriction>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ChemicalRestriction {
    #[validate(length(min = 1, max = 100, message = "Regulation name is required"))]
    pub regulation: String,
    #[validate(length(min = 1, max = 100, message = "Jurisdiction is required"))]
    pub jurisdiction: String,
    pub restriction_type: RestrictionType,
    #[validate(range(min = 0.0, message = "Threshold must be positive"))]
    pub threshold: Option<f64>,
    pub effective_date: DateTime<Utc>,
    #[validate(length(min = 1, max = 500, message = "Description is required"))]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestrictionType {
    Banned,
    Restricted,
    RequiresAuthorization,
    RequiresNotification,
    RequiresTesting,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CASValidation {
    pub is_valid: bool,
    pub normalized_cas: Option<String>,
    pub validation_errors: Vec<String>,
    #[validate(length(max = 255))]
    pub chemical_name: Option<String>,
    #[validate(range(min = 0.0, max = 1.0, message = "Confidence must be between 0.0 and 1.0"))]
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseUpdateResult {
    pub updated_substances: u32,
    pub new_substances: u32,
    pub removed_substances: u32,
    pub update_timestamp: DateTime<Utc>,
    #[validate(length(min = 1, max = 100, message = "Source is required"))]
    pub source: String,
    #[validate(length(min = 1, max = 50, message = "Version is required"))]
    pub version: String,
}

// Custom validation function
fn validate_cas_number(cas_number: &str) -> Result<(), ValidationError> {
    if !ChemicalSubstance::validate_cas_format(cas_number) {
        return Err(ValidationError::new("invalid_cas_format"));
    }
    Ok(())
}

impl ChemicalSubstance {
    /// Creates a new chemical substance with the given CAS number and name
    pub fn new(cas_number: String, chemical_name: String) -> Result<Self, String> {
        if !Self::validate_cas_format(&cas_number) {
            return Err(format!("Invalid CAS number format: {}", cas_number));
        }
        
        let mut substance = Self::default();
        substance.cas_number = cas_number;
        substance.chemical_name = chemical_name;
        substance.last_updated = Utc::now();
        
        Ok(substance)
    }
    
    /// Validates CAS number format
    pub fn validate_cas_format(cas_number: &str) -> bool {
        // CAS number format: XXXXXX-XX-X where X is a digit
        let parts: Vec<&str> = cas_number.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        
        // Check format: 2-7 digits, 2 digits, 1 digit
        if parts[0].len() < 2 || parts[0].len() > 7 || parts[1].len() != 2 || parts[2].len() != 1 {
            return false;
        }
        
        // Check all parts are numeric
        parts.iter().all(|part| part.chars().all(|c| c.is_ascii_digit()))
    }
    
    /// Calculates the check digit for a CAS number
    pub fn calculate_check_digit(cas_number: &str) -> Option<u8> {
        let digits: String = cas_number.replace('-', "");
        if digits.len() < 3 {
            return None;
        }
        
        let mut sum = 0;
        let digits_vec: Vec<u32> = digits.chars()
            .filter_map(|c| c.to_digit(10))
            .collect();
        
        // Calculate check digit (last digit)
        for (i, &digit) in digits_vec[..digits_vec.len()-1].iter().rev().enumerate() {
            sum += digit * (i as u32 + 1);
        }
        
        Some((sum % 10) as u8)
    }
    
    /// Validates the CAS number including check digit
    pub fn is_valid_cas(&self) -> bool {
        Self::validate_cas_format(&self.cas_number) &&
        Self::calculate_check_digit(&self.cas_number)
            .map(|check| check == self.cas_number.chars().last().unwrap().to_digit(10).unwrap() as u8)
            .unwrap_or(false)
    }
    
    /// Sets the PFAS classification for this substance
    pub fn set_pfas_classification(&mut self, classification: PFASClassification) {
        self.is_pfas = classification.is_pfas;
        self.pfas_classification = Some(classification);
        self.last_updated = Utc::now();
    }
    
    /// Checks if this substance requires regulatory reporting
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
    
    /// Gets active restrictions for this substance
    pub fn active_restrictions(&self) -> Vec<&ChemicalRestriction> {
        let now = Utc::now();
        self.regulatory_status.restrictions
            .iter()
            .filter(|restriction| restriction.effective_date <= now)
            .collect()
    }
    
    /// Checks if the substance is banned in any jurisdiction
    pub fn is_banned(&self) -> bool {
        self.regulatory_status.restrictions
            .iter()
            .any(|r| matches!(r.restriction_type, RestrictionType::Banned))
    }
    
    /// Gets the PFAS confidence score if available
    pub fn pfas_confidence(&self) -> Option<f64> {
        self.pfas_classification.as_ref().map(|c| c.confidence)
    }
    
    /// Updates regulatory status with new information
    pub fn update_regulatory_status(&mut self, status: RegulatoryStatus) {
        self.regulatory_status = status;
        self.last_updated = Utc::now();
    }
}

impl Default for ChemicalSubstance {
    fn default() -> Self {
        Self {
            cas_number: String::new(),
            chemical_name: String::new(),
            molecular_formula: None,
            molecular_weight: None,
            is_pfas: false,
            pfas_classification: None,
            regulatory_status: RegulatoryStatus {
                regulatory_lists: Vec::new(),
                reporting_requirements: Vec::new(),
                restrictions: Vec::new(),
                last_updated: Utc::now(),
            },
            last_updated: Utc::now(),
        }
    }
}

impl PFASClassification {
    /// Creates a new PFAS classification
    pub fn new(
        is_pfas: bool,
        confidence: f64,
        classification_source: String,
    ) -> Self {
        Self {
            is_pfas,
            confidence,
            regulatory_lists: Vec::new(),
            reporting_requirements: Vec::new(),
            classification_source,
            classification_date: Utc::now(),
        }
    }
    
    /// Checks if the classification is high confidence (>= 0.8)
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.8
    }
    
    /// Adds a regulatory list to the classification
    pub fn add_regulatory_list(&mut self, list: RegulatoryList) {
        self.regulatory_lists.push(list);
    }
    
    /// Adds a reporting requirement to the classification
    pub fn add_reporting_requirement(&mut self, requirement: ReportingRequirement) {
        self.reporting_requirements.push(requirement);
    }
}

impl CASValidation {
    /// Creates a new CAS validation result
    pub fn new(cas_number: &str) -> Self {
        let is_valid = ChemicalSubstance::validate_cas_format(cas_number);
        let normalized_cas = if is_valid {
            Some(cas_number.to_string())
        } else {
            None
        };
        
        let mut validation_errors = Vec::new();
        if !is_valid {
            validation_errors.push("Invalid CAS number format".to_string());
        }
        
        Self {
            is_valid,
            normalized_cas,
            validation_errors,
            chemical_name: None,
            confidence: if is_valid { 1.0 } else { 0.0 },
        }
    }
    
    /// Sets the chemical name for this validation
    pub fn with_chemical_name(mut self, name: String) -> Self {
        self.chemical_name = Some(name);
        self
    }
    
    /// Sets the confidence score for this validation
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
    
    /// Adds a validation error
    pub fn add_error(&mut self, error: String) {
        self.validation_errors.push(error);
        self.is_valid = false;
        self.confidence = 0.0;
    }
}