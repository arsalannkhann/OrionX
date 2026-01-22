//! Property-based tests for Elementa core domain models
//! 
//! This module contains property-based tests that validate universal properties
//! across all core domain models, focusing on serialization round-trip consistency
//! and data integrity guarantees.

use proptest::prelude::*;
use proptest::option;
use chrono::{DateTime, Utc, TimeZone};
use uuid::Uuid;

use crate::{
    SupplierRecord, ContactInfo, Address, SupplierRelationship, ComplianceHistoryEntry,
    ComplianceStatus, CommunicationPreferences, TechnicalLevel, ResponseFormat, RiskProfile,
    RiskLevel, Component, ComponentSpecifications, Dimensions, MaterialType,
    ComplianceRecord, CASRecord, ExtractionMethod, TestResult, TestType, Certification, CertificationType,
    ValidationStatus, DocumentReference, AuditEntry, AuditAction,
    AuditDetails,
};

// Import the correct regulatory types from compliance module
use crate::compliance::{RegulatoryStatus, RegulatoryList, ReportingRequirement};

// Property test generators for primitive types and common structures

prop_compose! {
    fn arb_datetime()(timestamp in 0i64..2147483647i64) -> DateTime<Utc> {
        Utc.timestamp_opt(timestamp, 0).unwrap()
    }
}

prop_compose! {
    fn arb_uuid()(bytes in prop::array::uniform16(0u8..)) -> Uuid {
        Uuid::from_bytes(bytes)
    }
}

prop_compose! {
    fn arb_cas_number()(
        first_part in 10..9999999u32,
        second_part in 10..99u32,
        third_part in 0..9u32
    ) -> String {
        format!("{}-{:02}-{}", first_part, second_part, third_part)
    }
}

prop_compose! {
    fn arb_email()(
        local in "[a-z]{3,10}",
        domain in "[a-z]{3,10}",
        tld in "[a-z]{2,4}"
    ) -> String {
        format!("{}@{}.{}", local, domain, tld)
    }
}

prop_compose! {
    fn arb_phone()(
        area in 100..1000u16,
        exchange in 100..1000u16,
        number in 1000..10000u16
    ) -> String {
        format!("+1-{}-{}-{}", area, exchange, number)
    }
}

// Generators for domain model components

prop_compose! {
    fn arb_address()(
        street in "[A-Za-z0-9 ]{10,50}",
        city in "[A-Za-z ]{3,30}",
        state in option::of("[A-Z]{2}"),
        postal_code in "[0-9]{5}",
        country in "[A-Z]{2,3}"
    ) -> Address {
        Address {
            street,
            city,
            state,
            postal_code,
            country,
        }
    }
}

prop_compose! {
    fn arb_contact_info()(
        primary_email in arb_email(),
        alternate_emails in prop::collection::vec(arb_email(), 0..3),
        contact_person in "[A-Za-z ]{5,50}",
        phone in option::of(arb_phone()),
        address in option::of(arb_address())
    ) -> ContactInfo {
        ContactInfo {
            primary_email,
            alternate_emails,
            contact_person,
            phone,
            address,
        }
    }
}

prop_compose! {
    fn arb_compliance_history_entry()(
        campaign_id in arb_uuid(),
        status in prop_oneof![
            Just(ComplianceStatus::NotStarted),
            Just(ComplianceStatus::InProgress),
            Just(ComplianceStatus::PartiallyComplete),
            Just(ComplianceStatus::Complete),
            Just(ComplianceStatus::NonCompliant),
            Just(ComplianceStatus::Escalated),
        ],
        response_time_days in option::of(1..30i32),
        completeness_score in 0.0..1.0f64,
        last_updated in arb_datetime()
    ) -> ComplianceHistoryEntry {
        ComplianceHistoryEntry {
            campaign_id,
            status,
            response_time_days,
            completeness_score,
            last_updated,
        }
    }
}

prop_compose! {
    fn arb_communication_preferences()(
        preferred_language in "[a-z]{2,5}",
        technical_level in prop_oneof![
            Just(TechnicalLevel::Basic),
            Just(TechnicalLevel::Intermediate),
            Just(TechnicalLevel::Advanced),
            Just(TechnicalLevel::Expert),
        ],
        response_format in prop_oneof![
            Just(ResponseFormat::Email),
            Just(ResponseFormat::Portal),
            Just(ResponseFormat::Phone),
            Just(ResponseFormat::Document),
        ],
        follow_up_frequency_days in 1..30i32
    ) -> CommunicationPreferences {
        CommunicationPreferences {
            preferred_language,
            technical_level,
            response_format,
            follow_up_frequency_days,
        }
    }
}

prop_compose! {
    fn arb_risk_profile()(
        compliance_risk in prop_oneof![
            Just(RiskLevel::Low),
            Just(RiskLevel::Medium),
            Just(RiskLevel::High),
            Just(RiskLevel::Critical),
        ],
        response_reliability in prop_oneof![
            Just(RiskLevel::Low),
            Just(RiskLevel::Medium),
            Just(RiskLevel::High),
            Just(RiskLevel::Critical),
        ],
        data_quality in prop_oneof![
            Just(RiskLevel::Low),
            Just(RiskLevel::Medium),
            Just(RiskLevel::High),
            Just(RiskLevel::Critical),
        ],
        overall_score in 0.0..1.0f64,
        last_assessed in arb_datetime()
    ) -> RiskProfile {
        RiskProfile {
            compliance_risk,
            response_reliability,
            data_quality,
            overall_score,
            last_assessed,
        }
    }
}

prop_compose! {
    fn arb_supplier_record()(
        id in arb_uuid(),
        name in "[A-Za-z0-9 ]{5,100}",
        contact_info in arb_contact_info(),
        relationship in prop_oneof![
            Just(SupplierRelationship::Strategic),
            Just(SupplierRelationship::Preferred),
            Just(SupplierRelationship::Standard),
            Just(SupplierRelationship::NewVendor),
            Just(SupplierRelationship::AtRisk),
        ],
        compliance_history in prop::collection::vec(arb_compliance_history_entry(), 0..5),
        communication_preferences in arb_communication_preferences(),
        risk_profile in arb_risk_profile(),
        created_at in arb_datetime(),
        updated_at in arb_datetime()
    ) -> SupplierRecord {
        SupplierRecord {
            id,
            name,
            contact_info,
            relationship,
            compliance_history,
            communication_preferences,
            risk_profile,
            created_at,
            updated_at,
        }
    }
}

prop_compose! {
    fn arb_dimensions()(
        length_mm in 0.1..1000.0f64,
        width_mm in 0.1..1000.0f64,
        height_mm in 0.1..1000.0f64
    ) -> Dimensions {
        Dimensions {
            length_mm,
            width_mm,
            height_mm,
        }
    }
}

prop_compose! {
    fn arb_component_specifications()(
        weight_grams in option::of(0.1..10000.0f64),
        dimensions in option::of(arb_dimensions()),
        color in option::of("[A-Za-z ]{3,20}"),
        finish in option::of("[A-Za-z ]{3,30}"),
        grade in option::of("[A-Za-z0-9]{1,10}"),
        certifications in prop::collection::vec("[A-Z0-9]{3,20}", 0..5),
        custom_properties in prop::collection::hash_map("[a-z_]{3,20}", "[A-Za-z0-9 ]{1,50}", 0..10)
    ) -> ComponentSpecifications {
        ComponentSpecifications {
            weight_grams,
            dimensions,
            color,
            finish,
            grade,
            certifications,
            custom_properties,
        }
    }
}

prop_compose! {
    fn arb_component()(
        id in arb_uuid(),
        part_number in "[A-Z0-9-]{5,20}",
        description in "[A-Za-z0-9 ]{10,100}",
        cas_numbers in prop::collection::vec(arb_cas_number(), 0..5),
        material_type in prop_oneof![
            Just(MaterialType::Metal),
            Just(MaterialType::Polymer),
            Just(MaterialType::Ceramic),
            Just(MaterialType::Composite),
            Just(MaterialType::Chemical),
            Just(MaterialType::Electronic),
            Just(MaterialType::Textile),
            "[A-Za-z ]{5,20}".prop_map(MaterialType::Other),
        ],
        supplier_id in arb_uuid(),
        specifications in arb_component_specifications(),
        created_at in arb_datetime(),
        updated_at in arb_datetime()
    ) -> Component {
        Component {
            id,
            part_number,
            description,
            cas_numbers,
            material_type,
            supplier_id,
            specifications,
            created_at,
            updated_at,
        }
    }
}

prop_compose! {
    fn arb_document_reference()(
        document_id in arb_uuid(),
        page in option::of(1..1000u32),
        section in option::of("[A-Za-z0-9 ]{3,50}"),
        extraction_timestamp in arb_datetime()
    ) -> DocumentReference {
        DocumentReference {
            document_id,
            page,
            section,
            extraction_timestamp,
        }
    }
}

prop_compose! {
    fn arb_regulatory_list()(
        source in "[A-Z]{2,10}",
        list_name in "[A-Za-z0-9 ]{10,50}",
        date_added in arb_datetime(),
        reporting_threshold in option::of(0.001..1000.0f64)
    ) -> RegulatoryList {
        RegulatoryList {
            source,
            list_name,
            date_added,
            reporting_threshold,
        }
    }
}

prop_compose! {
    fn arb_reporting_requirement()(
        regulation in "[A-Z]{2,20}",
        deadline in arb_datetime(),
        threshold in option::of(0.001..1000.0f64),
        reporting_format in "[A-Z]{2,20}"
    ) -> ReportingRequirement {
        ReportingRequirement {
            regulation,
            deadline,
            threshold,
            reporting_format,
        }
    }
}

prop_compose! {
    fn arb_regulatory_status()(
        regulatory_lists in prop::collection::vec(arb_regulatory_list(), 0..3),
        reporting_requirements in prop::collection::vec(arb_reporting_requirement(), 0..3),
        last_updated in arb_datetime()
    ) -> RegulatoryStatus {
        RegulatoryStatus {
            regulatory_lists,
            reporting_requirements,
            last_updated,
        }
    }
}

prop_compose! {
    fn arb_cas_record()(
        cas_number in arb_cas_number(),
        chemical_name in "[A-Za-z0-9 ]{5,50}",
        is_pfas in any::<bool>(),
        confidence in 0.0..1.0f64,
        regulatory_status in arb_regulatory_status(),
        source_document in arb_document_reference(),
        extraction_method in prop_oneof![
            Just(ExtractionMethod::VLMAutomatic),
            Just(ExtractionMethod::OCRProcessing),
            Just(ExtractionMethod::ManualEntry),
            Just(ExtractionMethod::DatabaseLookup),
        ],
        created_at in arb_datetime()
    ) -> CASRecord {
        CASRecord {
            cas_number,
            chemical_name,
            is_pfas,
            confidence,
            regulatory_status,
            source_document,
            extraction_method,
            created_at,
        }
    }
}

prop_compose! {
    fn arb_test_result()(
        test_type in prop_oneof![
            Just(TestType::PFASConcentration),
            Just(TestType::ChemicalComposition),
            Just(TestType::MaterialSafety),
            Just(TestType::EnvironmentalImpact),
            "[A-Za-z ]{5,20}".prop_map(TestType::Other),
        ],
        result_value in 0.0..1000.0f64,
        unit in "[a-z/%]{1,10}",
        detection_limit in option::of(0.001..10.0f64),
        test_method in "[A-Z0-9-]{5,20}",
        test_date in arb_datetime(),
        laboratory in "[A-Za-z ]{10,50}",
        certificate_number in option::of("[A-Z0-9-]{5,20}"),
        source_document in arb_document_reference()
    ) -> TestResult {
        TestResult {
            test_type,
            result_value,
            unit,
            detection_limit,
            test_method,
            test_date,
            laboratory,
            certificate_number,
            source_document,
        }
    }
}

prop_compose! {
    fn arb_certification()(
        certification_type in prop_oneof![
            Just(CertificationType::ISO14001),
            Just(CertificationType::REACH),
            Just(CertificationType::RoHS),
            Just(CertificationType::PfasFree),
            Just(CertificationType::MaterialSafety),
            "[A-Za-z0-9 ]{5,20}".prop_map(CertificationType::Other),
        ],
        issuing_body in "[A-Za-z ]{10,50}",
        certificate_number in "[A-Z0-9-]{5,20}",
        issue_date in arb_datetime(),
        expiry_date in option::of(arb_datetime()),
        scope in "[A-Za-z0-9 ]{20,100}",
        source_document in arb_document_reference()
    ) -> Certification {
        Certification {
            certification_type,
            issuing_body,
            certificate_number,
            issue_date,
            expiry_date,
            scope,
            source_document,
        }
    }
}

prop_compose! {
    fn arb_audit_entry()(
        id in arb_uuid(),
        timestamp in arb_datetime(),
        action in prop_oneof![
            Just(AuditAction::DocumentUploaded),
            Just(AuditAction::DocumentProcessed),
            Just(AuditAction::DataExtracted),
            Just(AuditAction::ComplianceRecordCreated),
            Just(AuditAction::ComplianceRecordUpdated),
            Just(AuditAction::EmailSent),
            Just(AuditAction::EmailReceived),
            Just(AuditAction::WorkflowStarted),
            Just(AuditAction::WorkflowCompleted),
            Just(AuditAction::EscalationCreated),
            Just(AuditAction::UserAction),
            Just(AuditAction::SystemAction),
        ],
        user_id in option::of(arb_uuid()),
        agent_id in option::of("[a-z_]{5,20}"),
        entity_type in "[A-Za-z]{5,20}",
        entity_id in arb_uuid(),
        metadata in prop::collection::hash_map("[a-z_]{3,15}", "[A-Za-z0-9 ]{1,30}", 0..5),
        source_document in option::of(arb_document_reference()),
        hash in "[a-f0-9]{64}",
        previous_hash in option::of("[a-f0-9]{64}"),
        created_at in arb_datetime()
    ) -> AuditEntry {
        let details = AuditDetails {
            entity_type,
            entity_id,
            changes: Vec::new(),
            metadata,
        };
        
        AuditEntry {
            id,
            timestamp,
            action,
            user_id,
            agent_id,
            details,
            source_document,
            hash,
            previous_hash,
            created_at,
        }
    }
}

prop_compose! {
    fn arb_compliance_record()(
        id in arb_uuid(),
        supplier_id in arb_uuid(),
        component_id in arb_uuid(),
        cas_records in prop::collection::vec(arb_cas_record(), 0..5),
        test_results in prop::collection::vec(arb_test_result(), 0..3),
        certifications in prop::collection::vec(arb_certification(), 0..3),
        submission_date in arb_datetime(),
        validation_status in prop_oneof![
            Just(ValidationStatus::Pending),
            Just(ValidationStatus::Valid),
            Just(ValidationStatus::Invalid),
            Just(ValidationStatus::RequiresReview),
            Just(ValidationStatus::Incomplete),
        ],
        audit_trail in prop::collection::vec(arb_audit_entry(), 0..5),
        created_at in arb_datetime(),
        updated_at in arb_datetime()
    ) -> ComplianceRecord {
        ComplianceRecord {
            id,
            supplier_id,
            component_id,
            cas_records,
            test_results,
            certifications,
            submission_date,
            validation_status,
            audit_trail,
            created_at,
            updated_at,
        }
    }
}

// Property test for serialization round-trip consistency
proptest! {
    /// **Property 1: Serialization round-trip consistency**
    /// **Validates: Requirements 1.2, 3.4**
    /// 
    /// For any valid instance of core domain models, serialization to JSON
    /// followed by deserialization produces an equivalent object with acceptable
    /// floating-point precision tolerance.
    #[test]
    fn property_serialization_round_trip_consistency_supplier_record(
        supplier in arb_supplier_record()
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&supplier)
            .expect("Serialization should succeed for valid SupplierRecord");
        
        // Deserialize back from JSON
        let deserialized: SupplierRecord = serde_json::from_str(&json)
            .expect("Deserialization should succeed for valid JSON");
        
        // Verify structural consistency (non-floating-point fields)
        prop_assert_eq!(supplier.id, deserialized.id);
        prop_assert_eq!(supplier.name, deserialized.name);
        prop_assert_eq!(supplier.contact_info.primary_email, deserialized.contact_info.primary_email);
        prop_assert_eq!(supplier.contact_info.contact_person, deserialized.contact_info.contact_person);
        prop_assert_eq!(supplier.relationship, deserialized.relationship);
        prop_assert_eq!(supplier.compliance_history.len(), deserialized.compliance_history.len());
        
        // Verify floating-point fields with tolerance
        let epsilon = 1e-10;
        prop_assert!((supplier.risk_profile.overall_score - deserialized.risk_profile.overall_score).abs() < epsilon,
                    "Overall score should be approximately equal: {} vs {}", 
                    supplier.risk_profile.overall_score, deserialized.risk_profile.overall_score);
        
        // Verify compliance history floating-point fields
        for (orig, deser) in supplier.compliance_history.iter().zip(deserialized.compliance_history.iter()) {
            prop_assert!((orig.completeness_score - deser.completeness_score).abs() < epsilon,
                        "Completeness score should be approximately equal: {} vs {}", 
                        orig.completeness_score, deser.completeness_score);
        }
    }

    #[test]
    fn property_serialization_round_trip_consistency_component(
        component in arb_component()
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&component)
            .expect("Serialization should succeed for valid Component");
        
        // Deserialize back from JSON
        let deserialized: Component = serde_json::from_str(&json)
            .expect("Deserialization should succeed for valid JSON");
        
        // Verify structural consistency (non-floating-point fields)
        prop_assert_eq!(component.id, deserialized.id);
        prop_assert_eq!(component.part_number, deserialized.part_number);
        prop_assert_eq!(component.description, deserialized.description);
        prop_assert_eq!(component.cas_numbers, deserialized.cas_numbers);
        prop_assert_eq!(component.supplier_id, deserialized.supplier_id);
        prop_assert_eq!(component.material_type, deserialized.material_type);
        
        // Verify floating-point fields with tolerance
        let epsilon = 1e-10;
        if let (Some(orig_weight), Some(deser_weight)) = (component.specifications.weight_grams, deserialized.specifications.weight_grams) {
            prop_assert!((orig_weight - deser_weight).abs() < epsilon,
                        "Weight should be approximately equal: {} vs {}", orig_weight, deser_weight);
        }
        
        if let (Some(orig_dims), Some(deser_dims)) = (&component.specifications.dimensions, &deserialized.specifications.dimensions) {
            prop_assert!((orig_dims.length_mm - deser_dims.length_mm).abs() < epsilon,
                        "Length should be approximately equal: {} vs {}", orig_dims.length_mm, deser_dims.length_mm);
            prop_assert!((orig_dims.width_mm - deser_dims.width_mm).abs() < epsilon,
                        "Width should be approximately equal: {} vs {}", orig_dims.width_mm, deser_dims.width_mm);
            prop_assert!((orig_dims.height_mm - deser_dims.height_mm).abs() < epsilon,
                        "Height should be approximately equal: {} vs {}", orig_dims.height_mm, deser_dims.height_mm);
        }
    }

    #[test]
    fn property_serialization_round_trip_consistency_compliance_record(
        record in arb_compliance_record()
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&record)
            .expect("Serialization should succeed for valid ComplianceRecord");
        
        // Deserialize back from JSON
        let deserialized: ComplianceRecord = serde_json::from_str(&json)
            .expect("Deserialization should succeed for valid JSON");
        
        // Verify structural consistency (non-floating-point fields)
        prop_assert_eq!(record.id, deserialized.id);
        prop_assert_eq!(record.supplier_id, deserialized.supplier_id);
        prop_assert_eq!(record.component_id, deserialized.component_id);
        prop_assert_eq!(record.cas_records.len(), deserialized.cas_records.len());
        prop_assert_eq!(record.validation_status, deserialized.validation_status);
        
        // Verify CAS records with floating-point tolerance
        let epsilon = 1e-10;
        for (orig_cas, deser_cas) in record.cas_records.iter().zip(deserialized.cas_records.iter()) {
            prop_assert_eq!(&orig_cas.cas_number, &deser_cas.cas_number);
            prop_assert_eq!(&orig_cas.chemical_name, &deser_cas.chemical_name);
            prop_assert_eq!(orig_cas.is_pfas, deser_cas.is_pfas);
            prop_assert!((orig_cas.confidence - deser_cas.confidence).abs() < epsilon,
                        "Confidence should be approximately equal: {} vs {}", 
                        orig_cas.confidence, deser_cas.confidence);
            
            // Check regulatory status floating-point fields
            for (orig_list, deser_list) in orig_cas.regulatory_status.regulatory_lists.iter()
                .zip(deser_cas.regulatory_status.regulatory_lists.iter()) {
                if let (Some(orig_threshold), Some(deser_threshold)) = (orig_list.reporting_threshold, deser_list.reporting_threshold) {
                    prop_assert!((orig_threshold - deser_threshold).abs() < epsilon,
                                "Reporting threshold should be approximately equal: {} vs {}", 
                                orig_threshold, deser_threshold);
                }
            }
            
            for (orig_req, deser_req) in orig_cas.regulatory_status.reporting_requirements.iter()
                .zip(deser_cas.regulatory_status.reporting_requirements.iter()) {
                if let (Some(orig_threshold), Some(deser_threshold)) = (orig_req.threshold, deser_req.threshold) {
                    prop_assert!((orig_threshold - deser_threshold).abs() < epsilon,
                                "Requirement threshold should be approximately equal: {} vs {}", 
                                orig_threshold, deser_threshold);
                }
            }
        }
    }

    #[test]
    fn property_serialization_round_trip_consistency_cas_record(
        cas_record in arb_cas_record()
    ) {
        // Serialize to JSON
        let json = serde_json::to_string(&cas_record)
            .expect("Serialization should succeed for valid CASRecord");
        
        // Deserialize back from JSON
        let deserialized: CASRecord = serde_json::from_str(&json)
            .expect("Deserialization should succeed for valid JSON");
        
        // Verify structural consistency (non-floating-point fields)
        prop_assert_eq!(&cas_record.cas_number, &deserialized.cas_number);
        prop_assert_eq!(&cas_record.chemical_name, &deserialized.chemical_name);
        prop_assert_eq!(cas_record.is_pfas, deserialized.is_pfas);
        prop_assert_eq!(cas_record.extraction_method, deserialized.extraction_method);
        
        // Verify floating-point fields with tolerance
        let epsilon = 1e-10;
        prop_assert!((cas_record.confidence - deserialized.confidence).abs() < epsilon,
                    "Confidence should be approximately equal: {} vs {}", 
                    cas_record.confidence, deserialized.confidence);
        
        // Verify regulatory status floating-point fields
        for (orig_list, deser_list) in cas_record.regulatory_status.regulatory_lists.iter()
            .zip(deserialized.regulatory_status.regulatory_lists.iter()) {
            if let (Some(orig_threshold), Some(deser_threshold)) = (orig_list.reporting_threshold, deser_list.reporting_threshold) {
                prop_assert!((orig_threshold - deser_threshold).abs() < epsilon,
                            "Reporting threshold should be approximately equal: {} vs {}", 
                            orig_threshold, deser_threshold);
            }
        }
        
        for (orig_req, deser_req) in cas_record.regulatory_status.reporting_requirements.iter()
            .zip(deserialized.regulatory_status.reporting_requirements.iter()) {
            if let (Some(orig_threshold), Some(deser_threshold)) = (orig_req.threshold, deser_req.threshold) {
                prop_assert!((orig_threshold - deser_threshold).abs() < epsilon,
                            "Requirement threshold should be approximately equal: {} vs {}", 
                            orig_threshold, deser_threshold);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::strategy::ValueTree;

    #[test]
    fn test_cas_number_generator_produces_valid_format() {
        let strategy = arb_cas_number();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..100 {
            let cas_number = strategy.new_tree(&mut runner).unwrap().current();
            
            // Verify CAS number format
            let parts: Vec<&str> = cas_number.split('-').collect();
            assert_eq!(parts.len(), 3, "CAS number should have 3 parts: {}", cas_number);
            assert!(parts[0].len() >= 2 && parts[0].len() <= 7, "First part should be 2-7 digits: {}", cas_number);
            assert_eq!(parts[1].len(), 2, "Second part should be 2 digits: {}", cas_number);
            assert_eq!(parts[2].len(), 1, "Third part should be 1 digit: {}", cas_number);
            
            // Verify all parts are numeric
            for part in parts {
                assert!(part.chars().all(|c: char| c.is_ascii_digit()), "All parts should be numeric: {}", cas_number);
            }
        }
    }

    #[test]
    fn test_email_generator_produces_valid_format() {
        let strategy = arb_email();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..100 {
            let email = strategy.new_tree(&mut runner).unwrap().current();
            
            // Basic email format validation
            assert!(email.contains('@'), "Email should contain @: {}", email);
            assert!(email.contains('.'), "Email should contain .: {}", email);
            
            let parts: Vec<&str> = email.split('@').collect();
            assert_eq!(parts.len(), 2, "Email should have exactly one @: {}", email);
            
            let domain_parts: Vec<&str> = parts[1].split('.').collect();
            assert!(domain_parts.len() >= 2, "Domain should have at least one dot: {}", email);
        }
    }

    #[test]
    fn test_supplier_record_generator_produces_valid_data() {
        let strategy = arb_supplier_record();
        let mut runner = proptest::test_runner::TestRunner::default();
        
        for _ in 0..10 {
            let supplier = strategy.new_tree(&mut runner).unwrap().current();
            
            // Verify basic constraints
            assert!(!supplier.name.is_empty(), "Supplier name should not be empty");
            assert!(!supplier.contact_info.primary_email.is_empty(), "Primary email should not be empty");
            assert!(!supplier.contact_info.contact_person.is_empty(), "Contact person should not be empty");
            assert!(supplier.risk_profile.overall_score >= 0.0 && supplier.risk_profile.overall_score <= 1.0, 
                   "Overall score should be between 0.0 and 1.0: {}", supplier.risk_profile.overall_score);
        }
    }
}