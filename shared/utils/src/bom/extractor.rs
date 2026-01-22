//! Supplier Extractor
//! 
//! Extracts and deduplicates suppliers from parsed BOM data.

use std::collections::HashMap;
use uuid::Uuid;

use super::parser::{ParsedBom, BomRow};
use elementa_models::{SupplierRecord, ContactInfo};

/// Extracted supplier with associated components
#[derive(Debug, Clone)]
pub struct ExtractedSupplier {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub components: Vec<ExtractedComponent>,
    pub source_rows: Vec<usize>,
    pub is_complete: bool,
    pub missing_fields: Vec<String>,
}

/// Extracted component information
#[derive(Debug, Clone)]
pub struct ExtractedComponent {
    pub part_number: String,
    pub description: Option<String>,
    pub material_type: Option<String>,
    pub cas_numbers: Vec<String>,
    pub source_row: usize,
}

/// Supplier extraction result
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub suppliers: Vec<ExtractedSupplier>,
    pub complete_count: usize,
    pub incomplete_count: usize,
    pub duplicate_count: usize,
    pub warnings: Vec<String>,
}

/// Supplier extractor with deduplication
pub struct SupplierExtractor {
    /// Require email for supplier to be considered complete
    require_email: bool,
    /// Require contact person
    require_contact: bool,
}

impl Default for SupplierExtractor {
    fn default() -> Self {
        Self {
            require_email: true,
            require_contact: false,
        }
    }
}

impl SupplierExtractor {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Configure email requirement
    pub fn with_email_required(mut self, required: bool) -> Self {
        self.require_email = required;
        self
    }
    
    /// Configure contact requirement
    pub fn with_contact_required(mut self, required: bool) -> Self {
        self.require_contact = required;
        self
    }
    
    /// Extract and deduplicate suppliers from parsed BOM
    pub fn extract(&self, bom: &ParsedBom) -> ExtractionResult {
        let mut supplier_map: HashMap<String, ExtractedSupplier> = HashMap::new();
        let mut warnings = Vec::new();
        let mut duplicate_count = 0;
        
        for row in &bom.rows {
            // Skip rows without supplier name
            let supplier_name = match &row.supplier_name {
                Some(name) if !name.is_empty() => name.clone(),
                _ => {
                    warnings.push(format!("Row {}: Missing supplier name, skipped", row.row_number));
                    continue;
                }
            };
            
            // Normalize supplier name for deduplication
            let normalized_name = self.normalize_supplier_name(&supplier_name);
            
            // Extract component data
            let component = self.extract_component(row);
            
            if let Some(existing) = supplier_map.get_mut(&normalized_name) {
                // Deduplicate - merge component into existing supplier
                duplicate_count += 1;
                existing.source_rows.push(row.row_number);
                
                if let Some(comp) = component {
                    existing.components.push(comp);
                }
                
                // Update contact info if missing
                if existing.email.is_none() && row.supplier_email.is_some() {
                    existing.email = row.supplier_email.clone();
                }
                if existing.contact_person.is_none() && row.contact_person.is_some() {
                    existing.contact_person = row.contact_person.clone();
                }
            } else {
                // New supplier
                let mut missing_fields = Vec::new();
                
                if self.require_email && row.supplier_email.is_none() {
                    missing_fields.push("email".to_string());
                }
                if self.require_contact && row.contact_person.is_none() {
                    missing_fields.push("contact_person".to_string());
                }
                
                let is_complete = missing_fields.is_empty();
                
                let supplier = ExtractedSupplier {
                    id: Uuid::new_v4(),
                    name: supplier_name.clone(),
                    email: row.supplier_email.clone(),
                    contact_person: row.contact_person.clone(),
                    components: component.into_iter().collect(),
                    source_rows: vec![row.row_number],
                    is_complete,
                    missing_fields,
                };
                
                supplier_map.insert(normalized_name, supplier);
            }
        }
        
        let suppliers: Vec<ExtractedSupplier> = supplier_map.into_values().collect();
        let complete_count = suppliers.iter().filter(|s| s.is_complete).count();
        let incomplete_count = suppliers.len() - complete_count;
        
        ExtractionResult {
            suppliers,
            complete_count,
            incomplete_count,
            duplicate_count,
            warnings,
        }
    }
    
    /// Convert extracted suppliers to domain model records
    pub fn to_supplier_records(&self, extraction: &ExtractionResult) -> Vec<SupplierRecord> {
        extraction.suppliers.iter()
            .filter(|s| s.is_complete)
            .map(|s| {
                SupplierRecord {
                    id: s.id,
                    name: s.name.clone(),
                    contact_info: ContactInfo {
                        primary_email: s.email.clone().unwrap_or_default(),
                        contact_person: s.contact_person.clone().unwrap_or_default(),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            })
            .collect()
    }
    
    /// Extract component from BOM row
    fn extract_component(&self, row: &BomRow) -> Option<ExtractedComponent> {
        let part_number = row.part_number.clone()?;
        
        Some(ExtractedComponent {
            part_number,
            description: row.description.clone(),
            material_type: row.material_type.clone(),
            cas_numbers: row.cas_numbers.clone(),
            source_row: row.row_number,
        })
    }
    
    /// Normalize supplier name for deduplication
    fn normalize_supplier_name(&self, name: &str) -> String {
        // Convert to lowercase, remove common suffixes, normalize whitespace
        let mut normalized = name.to_lowercase();
        
        // Remove common company suffixes
        for suffix in &[" inc", " inc.", " llc", " ltd", " ltd.", " corp", " corp.", " co", " co."] {
            if normalized.ends_with(suffix) {
                normalized = normalized[..normalized.len() - suffix.len()].to_string();
            }
        }
        
        // Normalize whitespace
        normalized.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bom::parser::BomFormat;
    
    #[test]
    fn test_supplier_deduplication() {
        let bom = ParsedBom {
            id: Uuid::new_v4(),
            filename: "test.csv".to_string(),
            format: BomFormat::Csv,
            rows: vec![
                BomRow {
                    row_number: 2,
                    supplier_name: Some("Acme Corp".to_string()),
                    supplier_email: Some("acme@example.com".to_string()),
                    contact_person: Some("John".to_string()),
                    part_number: Some("PN-001".to_string()),
                    description: Some("Widget".to_string()),
                    material_type: None,
                    cas_numbers: vec![],
                    raw_data: Default::default(),
                },
                BomRow {
                    row_number: 3,
                    supplier_name: Some("ACME CORP".to_string()), // Duplicate
                    supplier_email: None,
                    contact_person: None,
                    part_number: Some("PN-002".to_string()),
                    description: Some("Gadget".to_string()),
                    material_type: None,
                    cas_numbers: vec![],
                    raw_data: Default::default(),
                },
            ],
            column_headers: vec![],
            total_rows: 2,
            parse_warnings: vec![],
        };
        
        let extractor = SupplierExtractor::new();
        let result = extractor.extract(&bom);
        
        // Should deduplicate to 1 supplier
        assert_eq!(result.suppliers.len(), 1);
        // With 2 components
        assert_eq!(result.suppliers[0].components.len(), 2);
        assert_eq!(result.duplicate_count, 1);
    }
}
