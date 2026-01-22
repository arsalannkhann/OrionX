//! Document Extraction Service
//! 
//! Orchestrates document processing and VLM extraction.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::pdf_processor::{PdfProcessor, CasMatch};


/// Stored document
#[derive(Debug, Clone)]
pub struct StoredDocument {
    pub id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub upload_date: String,
    pub status: String,
    pub data: Vec<u8>,
    pub extraction: Option<ExtractionResult>,
}

/// Extraction result
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    pub cas_numbers: Vec<CasExtractionResponse>,
    pub test_results: Vec<TestResultResponse>,
    pub certifications: Vec<CertificationResponse>,
    pub overall_confidence: f64,
    pub uncertainties: Vec<UncertaintyResponse>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CasExtractionResponse {
    pub cas_number: String,
    pub confidence: f64,
    pub context: String,
    pub page: Option<usize>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TestResultResponse {
    pub test_name: String,
    pub result: String,
    pub unit: Option<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CertificationResponse {
    pub name: String,
    pub issuer: Option<String>,
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct UncertaintyResponse {
    pub field: String,
    pub reason: String,
    pub alternatives: Vec<String>,
}

/// Document extractor service
#[derive(Clone)]
pub struct DocumentExtractor {
    documents: Arc<RwLock<HashMap<Uuid, StoredDocument>>>,
    pdf_processor: Arc<PdfProcessor>,
}

impl DocumentExtractor {
    pub fn new() -> Self {
        Self {
            documents: Arc::new(RwLock::new(HashMap::new())),
            pdf_processor: Arc::new(PdfProcessor::new()),
        }
    }
    
    /// Store uploaded document
    pub async fn store_document(&self, filename: &str, file_type: &str, data: &[u8]) -> Result<Uuid> {
        let id = Uuid::new_v4();
        let doc = StoredDocument {
            id,
            filename: filename.to_string(),
            file_type: file_type.to_string(),
            upload_date: chrono::Utc::now().to_rfc3339(),
            status: "uploaded".to_string(),
            data: data.to_vec(),
            extraction: None,
        };
        
        let mut docs = self.documents.write().await;
        docs.insert(id, doc);
        
        Ok(id)
    }
    
    /// Get document by ID
    pub async fn get_document(&self, id: Uuid) -> Result<Option<StoredDocument>> {
        let docs = self.documents.read().await;
        Ok(docs.get(&id).cloned())
    }
    
    /// Extract data from document
    pub async fn extract(&self, id: Uuid) -> Result<ExtractionResult> {
        let mut docs = self.documents.write().await;
        let doc = docs.get_mut(&id)
            .ok_or_else(|| anyhow::anyhow!("Document not found"))?;
        
        // Update status
        doc.status = "processing".to_string();
        
        // Determine extraction method based on file type
        let extraction = if doc.file_type.contains("pdf") {
            self.extract_from_pdf(&doc.data).await?
        } else {
            // For images, use VLM directly
            // For now, return empty result
            self.create_empty_result()
        };
        
        doc.extraction = Some(extraction.clone());
        doc.status = "extracted".to_string();
        
        Ok(extraction)
    }
    
    /// Extract from PDF
    async fn extract_from_pdf(&self, data: &[u8]) -> Result<ExtractionResult> {
        // First, extract text and CAS numbers using regex
        let pdf_content = self.pdf_processor.extract(data)?;
        let cas_matches = self.pdf_processor.extract_cas_numbers(&pdf_content.text);
        
        // Convert to response format
        let cas_numbers: Vec<CasExtractionResponse> = cas_matches.into_iter()
            .map(|m| {
                let cas_number = m.cas_number.clone();
                let confidence = self.validate_cas_confidence(&m);
                CasExtractionResponse {
                    cas_number,
                    confidence,
                    context: m.context,
                    page: Some(1),
                }
            })
            .collect();
        
        // Calculate overall confidence
        let overall_confidence = if cas_numbers.is_empty() {
            0.5 // No CAS numbers found - medium confidence
        } else {
            cas_numbers.iter().map(|c| c.confidence).sum::<f64>() / cas_numbers.len() as f64
        };
        
        // Flag uncertainties
        let mut uncertainties = Vec::new();
        for cas in &cas_numbers {
            if cas.confidence < 0.7 {
                uncertainties.push(UncertaintyResponse {
                    field: "cas_number".to_string(),
                    reason: format!("Low confidence extraction: {}", cas.cas_number),
                    alternatives: Vec::new(),
                });
            }
        }
        
        Ok(ExtractionResult {
            cas_numbers,
            test_results: Vec::new(), // Would need VLM for structured test results
            certifications: Vec::new(),
            overall_confidence,
            uncertainties,
        })
    }
    
    /// Validate CAS and calculate confidence
    fn validate_cas_confidence(&self, cas_match: &CasMatch) -> f64 {
        // Basic CAS checksum validation
        let parts: Vec<&str> = cas_match.cas_number.split('-').collect();
        if parts.len() != 3 {
            return 0.3;
        }
        
        let check_digit: u32 = match parts[2].parse() {
            Ok(d) => d,
            Err(_) => return 0.3,
        };
        
        let digits: String = format!("{}{}", parts[0], parts[1]);
        let sum: u32 = digits.chars()
            .rev()
            .enumerate()
            .filter_map(|(i, c)| c.to_digit(10).map(|d| d * (i as u32 + 1)))
            .sum();
        
        if sum % 10 == check_digit {
            0.95 // High confidence - checksum valid
        } else {
            0.5 // Medium confidence - format valid but checksum failed
        }
    }
    
    fn create_empty_result(&self) -> ExtractionResult {
        ExtractionResult {
            cas_numbers: Vec::new(),
            test_results: Vec::new(),
            certifications: Vec::new(),
            overall_confidence: 0.0,
            uncertainties: vec![UncertaintyResponse {
                field: "document".to_string(),
                reason: "Unsupported document format".to_string(),
                alternatives: vec!["Upload PDF or image file".to_string()],
            }],
        }
    }
}

impl Default for DocumentExtractor {
    fn default() -> Self {
        Self::new()
    }
}
