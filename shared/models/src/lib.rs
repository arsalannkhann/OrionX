//! # Elementa Core Domain Models
//! 
//! This module contains the core domain models for the Elementa supply chain compliance system.
//! All models implement proper serialization/deserialization with serde and validation with the validator crate.
//! 
//! ## Key Models
//! 
//! - **SupplierRecord**: Represents a supplier with contact information, compliance history, and risk profile
//! - **Component**: Represents a component or part with CAS numbers and specifications
//! - **ComplianceRecord**: Represents compliance data for a supplier-component pair
//! - **CASRecord**: Represents a chemical substance with CAS number and PFAS classification
//! - **ChemicalSubstance**: Represents detailed chemical information with regulatory status
//! 
//! ## Validation
//! 
//! All models include comprehensive validation rules:
//! - Email format validation
//! - CAS number format validation
//! - Range validation for numeric fields
//! - Length validation for string fields
//! 
//! ## Utility Methods
//! 
//! Models include utility methods for common operations:
//! - Risk profile calculation
//! - CAS number management
//! - Validation status updates
//! - PFAS classification handling

pub mod supplier;
pub mod component;
pub mod compliance;
pub mod document;
pub mod workflow;
pub mod audit;
pub mod email;
pub mod chemical;

#[cfg(test)]
pub mod property_tests;

pub use supplier::*;
pub use component::*;
pub use compliance::{
    ComplianceRecord, CASRecord, ExtractionMethod, TestResult, TestType,
    Certification, CertificationType, ValidationStatus
};
pub use document::*;
pub use workflow::*;
pub use audit::*;
pub use email::*;
pub use chemical::{
    ChemicalSubstance, PFASClassification, ChemicalRestriction, RestrictionType,
    CASValidation, DatabaseUpdateResult,
    RegulatoryStatus as ChemicalRegulatoryStatus,
    RegulatoryList as ChemicalRegulatoryList,
    ReportingRequirement as ChemicalReportingRequirement
};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    #[test]
    fn test_supplier_creation() {
        let supplier = SupplierRecord::default();
        assert!(!supplier.id.to_string().is_empty());
        assert_eq!(supplier.relationship, SupplierRelationship::Standard);
    }

    #[test]
    fn test_cas_number_validation() {
        // Valid CAS numbers
        assert!(ChemicalSubstance::validate_cas_format("7732-18-5")); // Water
        assert!(ChemicalSubstance::validate_cas_format("64-17-5"));   // Ethanol
        
        // Invalid CAS numbers
        assert!(!ChemicalSubstance::validate_cas_format("123-45"));
        assert!(!ChemicalSubstance::validate_cas_format("abc-de-f"));
    }

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(
            AuditAction::DocumentUploaded,
            "test_entity".to_string(),
            uuid::Uuid::new_v4(),
            None,
            Some("test_agent".to_string()),
        );
        
        assert!(!entry.hash.is_empty());
        assert!(entry.verify_integrity());
    }

    #[test]
    fn test_supplier_new() {
        let supplier = SupplierRecord::new(
            "Test Supplier".to_string(),
            "test@example.com".to_string(),
            "John Doe".to_string(),
        );
        
        assert_eq!(supplier.name, "Test Supplier");
        assert_eq!(supplier.contact_info.primary_email, "test@example.com");
        assert_eq!(supplier.contact_info.contact_person, "John Doe");
    }

    #[test]
    fn test_supplier_risk_profile_update() {
        let mut supplier = SupplierRecord::default();
        
        // Add some compliance history
        supplier.add_compliance_history(ComplianceHistoryEntry {
            campaign_id: Uuid::new_v4(),
            status: ComplianceStatus::Complete,
            response_time_days: Some(2),
            completeness_score: 0.95,
            last_updated: Utc::now(),
        });
        
        supplier.add_compliance_history(ComplianceHistoryEntry {
            campaign_id: Uuid::new_v4(),
            status: ComplianceStatus::Complete,
            response_time_days: Some(3),
            completeness_score: 0.90,
            last_updated: Utc::now(),
        });
        
        // Risk profile should be updated to Low
        assert_eq!(supplier.risk_profile.compliance_risk, RiskLevel::Low);
        assert_eq!(supplier.risk_profile.response_reliability, RiskLevel::Low);
        assert!(!supplier.is_high_risk());
    }

    #[test]
    fn test_component_cas_number_management() {
        let mut component = Component::new(
            "PART-001".to_string(),
            "Test Component".to_string(),
            Uuid::new_v4(),
        );
        
        // Add valid CAS number
        assert!(component.add_cas_number("7732-18-5".to_string()).is_ok());
        assert!(component.has_cas_numbers());
        assert_eq!(component.cas_numbers.len(), 1);
        
        // Try to add invalid CAS number
        assert!(component.add_cas_number("invalid-cas".to_string()).is_err());
        assert_eq!(component.cas_numbers.len(), 1); // Should still be 1
        
        // Remove CAS number
        component.remove_cas_number("7732-18-5");
        assert!(!component.has_cas_numbers());
    }

    #[test]
    fn test_compliance_record_validation_status() {
        let mut record = ComplianceRecord::new(Uuid::new_v4(), Uuid::new_v4());
        
        // Initially pending
        assert_eq!(record.validation_status, ValidationStatus::Pending);
        
        // Update validation status manually to see the logic
        record.update_validation_status();
        assert_eq!(record.validation_status, ValidationStatus::Incomplete);
        
        // Add high confidence CAS record
        let cas_record = CASRecord::new(
            "7732-18-5".to_string(),
            "Water".to_string(),
            false,
            0.95,
            DocumentReference {
                document_id: Uuid::new_v4(),
                page: Some(1),
                section: None,
                extraction_timestamp: Utc::now(),
            },
            ExtractionMethod::VLMAutomatic,
        );
        
        record.add_cas_record(cas_record);
        assert_eq!(record.validation_status, ValidationStatus::Valid);
        assert_eq!(record.overall_confidence(), 0.95);
    }

    #[test]
    fn test_chemical_substance_creation() {
        let substance = ChemicalSubstance::new(
            "7732-18-5".to_string(),
            "Water".to_string(),
        );
        
        assert!(substance.is_ok());
        let substance = substance.unwrap();
        assert_eq!(substance.cas_number, "7732-18-5");
        assert_eq!(substance.chemical_name, "Water");
        assert!(substance.is_valid_cas());
    }

    #[test]
    fn test_chemical_substance_invalid_cas() {
        let substance = ChemicalSubstance::new(
            "invalid-cas".to_string(),
            "Invalid Chemical".to_string(),
        );
        
        assert!(substance.is_err());
    }

    #[test]
    fn test_pfas_classification() {
        let mut classification = PFASClassification::new(
            true,
            0.85,
            "EPA Database".to_string(),
        );
        
        assert!(classification.is_pfas);
        assert!(classification.is_high_confidence());
        
        classification.add_regulatory_list(crate::chemical::RegulatoryList {
            source: "EPA".to_string(),
            list_name: "PFAS Master List".to_string(),
            date_added: Utc::now(),
            reporting_threshold: Some(1.0),
            list_version: "2024.1".to_string(),
        });
        
        assert_eq!(classification.regulatory_lists.len(), 1);
    }

    #[test]
    fn test_cas_validation() {
        let validation = CASValidation::new("7732-18-5");
        assert!(validation.is_valid);
        assert_eq!(validation.confidence, 1.0);
        assert!(validation.validation_errors.is_empty());
        
        let validation = CASValidation::new("invalid-cas");
        assert!(!validation.is_valid);
        assert_eq!(validation.confidence, 0.0);
        assert!(!validation.validation_errors.is_empty());
    }

    #[test]
    fn test_contact_info_phone_validation() {
        let mut contact = ContactInfo::default();
        
        // Valid phone numbers
        assert!(contact.set_phone(Some("+1-555-123-4567".to_string())).is_ok());
        assert!(contact.set_phone(Some("555 123 4567".to_string())).is_ok());
        assert!(contact.set_phone(Some("(555) 123-4567".to_string())).is_ok());
        
        // Invalid phone number
        assert!(contact.set_phone(Some("abc-def-ghij".to_string())).is_err());
        
        // None is valid
        assert!(contact.set_phone(None).is_ok());
    }
}