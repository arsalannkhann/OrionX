//! Chemical Service
//! 
//! Core business logic for CAS validation and PFAS classification.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Chemical substance data
#[derive(Debug, Clone)]
pub struct Chemical {
    pub cas_number: String,
    pub chemical_name: String,
    pub molecular_formula: Option<String>,
    pub molecular_weight: Option<f64>,
    pub is_pfas: bool,
    pub pfas_classification: Option<PfasClassification>,
    pub regulatory_status: Vec<RegulatoryStatus>,
}

/// PFAS classification details
#[derive(Debug, Clone)]
pub struct PfasClassification {
    pub is_pfas: bool,
    pub confidence: f64,
    pub source: String,
    pub regulatory_lists: Vec<RegulatoryList>,
    pub reporting_requirements: Vec<ReportingRequirement>,
}

/// Regulatory list information
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RegulatoryList {
    pub source: String,
    pub list_name: String,
    pub date_added: String,
}

/// Reporting requirement
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ReportingRequirement {
    pub regulation: String,
    pub description: String,
    pub threshold: Option<String>,
}

/// Regulatory status
#[derive(Debug, Clone)]
pub struct RegulatoryStatus {
    pub source: String,
    pub status: String,
    pub reporting_threshold: Option<String>,
}

/// CAS validation result
#[derive(Debug, Clone)]
pub struct CasValidation {
    pub is_valid: bool,
    pub format_valid: bool,
    pub checksum_valid: bool,
    pub normalized: String,
    pub errors: Vec<String>,
}

/// PFAS statistics
#[derive(Debug, Clone)]
pub struct PfasStats {
    pub total: usize,
    pub sources: Vec<SourceStats>,
    pub last_sync: String,
}

#[derive(Debug, Clone)]
pub struct SourceStats {
    pub name: String,
    pub count: usize,
    pub last_updated: String,
}

/// Sync result
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub new_count: usize,
    pub updated_count: usize,
    pub errors: Vec<String>,
}

/// Chemical database service
#[derive(Clone)]
pub struct ChemicalService {
    cache: Arc<RwLock<HashMap<String, Chemical>>>,
    pfas_list: Arc<RwLock<Vec<String>>>,
}

impl ChemicalService {
    pub fn new() -> Self {
        let mut cache = HashMap::new();
        let mut pfas_list = Vec::new();
        
        // Load known PFAS substances (subset for demo)
        let known_pfas = vec![
            ("335-67-1", "Perfluorooctanoic acid (PFOA)", true),
            ("1763-23-1", "Perfluorooctane sulfonic acid (PFOS)", true),
            ("375-73-5", "Perfluorobutane sulfonic acid (PFBS)", true),
            ("355-46-4", "Perfluorohexane sulfonic acid (PFHxS)", true),
            // Common non-PFAS chemicals
            ("7732-18-5", "Water", false),
            ("7647-14-5", "Sodium chloride", false),
            ("50-00-0", "Formaldehyde", false),
        ];
        
        for (cas, name, is_pfas) in known_pfas {
            if is_pfas {
                pfas_list.push(cas.to_string());
            }
            cache.insert(cas.to_string(), Chemical {
                cas_number: cas.to_string(),
                chemical_name: name.to_string(),
                molecular_formula: None,
                molecular_weight: None,
                is_pfas,
                pfas_classification: if is_pfas {
                    Some(PfasClassification {
                        is_pfas: true,
                        confidence: 1.0,
                        source: "EPA PFAS Master List".to_string(),
                        regulatory_lists: vec![RegulatoryList {
                            source: "EPA".to_string(),
                            list_name: "TSCA PFAS List".to_string(),
                            date_added: "2024-01-01".to_string(),
                        }],
                        reporting_requirements: vec![ReportingRequirement {
                            regulation: "TSCA Section 8(a)(7)".to_string(),
                            description: "PFAS Reporting Requirement".to_string(),
                            threshold: None,
                        }],
                    })
                } else {
                    None
                },
                regulatory_status: vec![],
            });
        }
        
        Self {
            cache: Arc::new(RwLock::new(cache)),
            pfas_list: Arc::new(RwLock::new(pfas_list)),
        }
    }
    
    /// Lookup chemical by CAS number
    pub async fn lookup(&self, cas_number: &str) -> Result<Option<Chemical>> {
        let normalized = self.normalize_cas(cas_number);
        let cache = self.cache.read().await;
        Ok(cache.get(&normalized).cloned())
    }
    
    /// Validate CAS number format and checksum
    pub fn validate_cas(&self, cas_number: &str) -> CasValidation {
        let normalized = self.normalize_cas(cas_number);
        let mut errors = Vec::new();
        
        // Check format
        let parts: Vec<&str> = normalized.split('-').collect();
        let format_valid = if parts.len() != 3 {
            errors.push("Invalid format: expected XXX-XX-X".to_string());
            false
        } else {
            let p1_valid = (2..=7).contains(&parts[0].len()) && parts[0].chars().all(|c| c.is_numeric());
            let p2_valid = parts[1].len() == 2 && parts[1].chars().all(|c| c.is_numeric());
            let p3_valid = parts[2].len() == 1 && parts[2].chars().all(|c| c.is_numeric());
            
            if !p1_valid {
                errors.push("First segment should be 2-7 digits".to_string());
            }
            if !p2_valid {
                errors.push("Second segment should be 2 digits".to_string());
            }
            if !p3_valid {
                errors.push("Third segment should be 1 digit (check digit)".to_string());
            }
            
            p1_valid && p2_valid && p3_valid
        };
        
        // Check checksum
        let checksum_valid = if format_valid {
            self.verify_cas_checksum(&normalized)
        } else {
            false
        };
        
        if format_valid && !checksum_valid {
            errors.push("Check digit verification failed".to_string());
        }
        
        CasValidation {
            is_valid: format_valid && checksum_valid,
            format_valid,
            checksum_valid,
            normalized,
            errors,
        }
    }
    
    /// Classify CAS number for PFAS status
    pub async fn classify_pfas(&self, cas_number: &str) -> Result<PfasClassification> {
        let normalized = self.normalize_cas(cas_number);
        let pfas_list = self.pfas_list.read().await;
        
        if pfas_list.contains(&normalized) {
            Ok(PfasClassification {
                is_pfas: true,
                confidence: 1.0,
                source: "EPA PFAS Master List".to_string(),
                regulatory_lists: vec![RegulatoryList {
                    source: "EPA".to_string(),
                    list_name: "TSCA PFAS List".to_string(),
                    date_added: "2024-01-01".to_string(),
                }],
                reporting_requirements: vec![ReportingRequirement {
                    regulation: "TSCA Section 8(a)(7)".to_string(),
                    description: "PFAS Reporting Requirement".to_string(),
                    threshold: None,
                }],
            })
        } else {
            Ok(PfasClassification {
                is_pfas: false,
                confidence: 0.9, // Not 100% confident it's not PFAS
                source: "Database lookup".to_string(),
                regulatory_lists: vec![],
                reporting_requirements: vec![],
            })
        }
    }
    
    /// Get PFAS statistics
    pub async fn get_pfas_stats(&self) -> PfasStats {
        let pfas_list = self.pfas_list.read().await;
        
        PfasStats {
            total: pfas_list.len(),
            sources: vec![
                SourceStats {
                    name: "EPA PFAS Master List".to_string(),
                    count: pfas_list.len(),
                    last_updated: "2024-01-01".to_string(),
                },
            ],
            last_sync: "2024-01-01T00:00:00Z".to_string(),
        }
    }
    
    /// Sync from external sources (EPA, OECD, etc.)
    pub async fn sync_from_sources(&self) -> Result<SyncResult> {
        // TODO: Implement actual EPA API integration
        Ok(SyncResult {
            new_count: 0,
            updated_count: 0,
            errors: vec!["External API integration not yet implemented".to_string()],
        })
    }
    
    /// Normalize CAS number format
    fn normalize_cas(&self, cas: &str) -> String {
        cas.chars()
            .filter(|c| c.is_numeric() || *c == '-')
            .collect()
    }
    
    /// Verify CAS check digit
    fn verify_cas_checksum(&self, cas: &str) -> bool {
        let parts: Vec<&str> = cas.split('-').collect();
        if parts.len() != 3 {
            return false;
        }
        
        let check_digit: u32 = match parts[2].parse() {
            Ok(d) => d,
            Err(_) => return false,
        };
        
        let digits: String = format!("{}{}", parts[0], parts[1]);
        
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
    use proptest::prelude::*;
    
    #[tokio::test]
    async fn test_pfas_classification() {
        let service = ChemicalService::new();
        
        // Known PFAS
        let pfoa = service.classify_pfas("335-67-1").await.unwrap();
        assert!(pfoa.is_pfas);
        assert_eq!(pfoa.confidence, 1.0);
        
        // Known non-PFAS
        let water = service.classify_pfas("7732-18-5").await.unwrap();
        assert!(!water.is_pfas);
    }
    
    #[test]
    fn test_cas_validation() {
        let service = ChemicalService::new();
        
        // Valid CAS numbers
        assert!(service.validate_cas("7732-18-5").is_valid);
        assert!(service.validate_cas("335-67-1").is_valid);
        
        // Invalid format
        assert!(!service.validate_cas("invalid").is_valid);
        assert!(!service.validate_cas("123-45").is_valid);
    }
    
    proptest! {
        /// Property 5: CAS Validation and PFAS Classification
        /// Valid format CAS numbers should normalize consistently
        #[test]
        fn prop_cas_normalization(
            p1 in "[0-9]{2,7}",
            p2 in "[0-9]{2}",
            p3 in "[0-9]{1}",
        ) {
            let cas = format!("{}-{}-{}", p1, p2, p3);
            let service = ChemicalService::new();
            let validation = service.validate_cas(&cas);
            
            // Normalized form should be consistent
            prop_assert_eq!(&validation.normalized, &cas);
            // Format should be valid
            prop_assert!(validation.format_valid);
        }
    }
}
