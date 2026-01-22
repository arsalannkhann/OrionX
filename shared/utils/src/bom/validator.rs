//! BOM Validator
//! 
//! Validates BOM data for completeness and correctness.

use super::parser::ParsedBom;

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

/// Single validation issue
#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub row: Option<usize>,
    pub field: Option<String>,
    pub message: String,
    pub suggestion: Option<String>,
}

/// Validation result for a BOM
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub issues: Vec<ValidationIssue>,
    pub summary: ValidationSummary,
}

/// Summary statistics for validation
#[derive(Debug, Clone)]
pub struct ValidationSummary {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub invalid_rows: usize,
    pub missing_suppliers: usize,
    pub missing_emails: usize,
    pub missing_parts: usize,
    pub invalid_cas_numbers: usize,
}

/// BOM validator
pub struct BomValidator {
    require_supplier: bool,
    require_email: bool,
    require_part_number: bool,
    validate_cas_format: bool,
}

impl Default for BomValidator {
    fn default() -> Self {
        Self {
            require_supplier: true,
            require_email: true,
            require_part_number: true,
            validate_cas_format: true,
        }
    }
}

impl BomValidator {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Validate parsed BOM
    pub fn validate(&self, bom: &ParsedBom) -> ValidationResult {
        let mut issues = Vec::new();
        let mut missing_suppliers = 0;
        let mut missing_emails = 0;
        let mut missing_parts = 0;
        let mut invalid_cas_numbers = 0;
        
        for row in &bom.rows {
            // Check supplier name
            if self.require_supplier && row.supplier_name.is_none() {
                missing_suppliers += 1;
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    row: Some(row.row_number),
                    field: Some("supplier_name".to_string()),
                    message: "Missing supplier name".to_string(),
                    suggestion: Some("Add supplier name to this row".to_string()),
                });
            }
            
            // Check email
            if self.require_email && row.supplier_email.is_none() {
                missing_emails += 1;
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    row: Some(row.row_number),
                    field: Some("supplier_email".to_string()),
                    message: "Missing supplier email".to_string(),
                    suggestion: Some("Add supplier email for compliance outreach".to_string()),
                });
            }
            
            // Check part number
            if self.require_part_number && row.part_number.is_none() {
                missing_parts += 1;
                issues.push(ValidationIssue {
                    severity: ValidationSeverity::Warning,
                    row: Some(row.row_number),
                    field: Some("part_number".to_string()),
                    message: "Missing part number".to_string(),
                    suggestion: Some("Add part number for component tracking".to_string()),
                });
            }
            
            // Validate CAS numbers
            if self.validate_cas_format {
                for cas in &row.cas_numbers {
                    if !self.is_valid_cas(cas) {
                        invalid_cas_numbers += 1;
                        issues.push(ValidationIssue {
                            severity: ValidationSeverity::Warning,
                            row: Some(row.row_number),
                            field: Some("cas_number".to_string()),
                            message: format!("Invalid CAS number format: {}", cas),
                            suggestion: Some("CAS format should be XXXXXXX-XX-X".to_string()),
                        });
                    }
                }
            }
        }
        
        let error_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Error).count();
        let warning_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count();
        let invalid_rows = bom.rows.iter()
            .filter(|r| r.supplier_name.is_none())
            .count();
        
        ValidationResult {
            is_valid: error_count == 0,
            error_count,
            warning_count,
            issues,
            summary: ValidationSummary {
                total_rows: bom.total_rows,
                valid_rows: bom.total_rows - invalid_rows,
                invalid_rows,
                missing_suppliers,
                missing_emails,
                missing_parts,
                invalid_cas_numbers,
            },
        }
    }
    
    /// Validate CAS number format
    fn is_valid_cas(&self, cas: &str) -> bool {
        let parts: Vec<&str> = cas.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        
        // First part: 2-7 digits
        if !(2..=7).contains(&parts[0].len()) || !parts[0].chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // Second part: 2 digits
        if parts[1].len() != 2 || !parts[1].chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // Third part: 1 digit (check digit)
        if parts[2].len() != 1 || !parts[2].chars().all(|c| c.is_numeric()) {
            return false;
        }
        
        // Optional: validate check digit
        self.validate_cas_checksum(cas)
    }
    
    /// Validate CAS check digit
    fn validate_cas_checksum(&self, cas: &str) -> bool {
        let parts: Vec<&str> = cas.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        
        let check_digit: u32 = match parts[2].parse() {
            Ok(d) => d,
            Err(_) => return false,
        };
        
        // Combine first two parts
        let digits: String = format!("{}{}", parts[0], parts[1]);
        
        // Calculate checksum
        let sum: u32 = digits.chars()
            .rev()
            .enumerate()
            .filter_map(|(i, c)| c.to_digit(10).map(|d| d * (i as u32 + 1)))
            .sum();
        
        sum % 10 == check_digit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_valid_cas_numbers() {
        let validator = BomValidator::new();
        
        // Valid CAS numbers
        assert!(validator.is_valid_cas("7732-18-5")); // Water
        assert!(validator.is_valid_cas("7647-14-5")); // Sodium chloride
        assert!(validator.is_valid_cas("50-00-0"));   // Formaldehyde
        
        // Invalid formats
        assert!(!validator.is_valid_cas("invalid"));
        assert!(!validator.is_valid_cas("123-45"));
        assert!(!validator.is_valid_cas("12345678-12-1")); // Too many digits
    }
}
