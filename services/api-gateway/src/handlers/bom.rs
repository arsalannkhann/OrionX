//! BOM Upload Handler
//! 
//! Handles file uploads for Bill of Materials processing.

use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;
use elementa_utils::bom::{BomParser, SupplierExtractor, BomValidator};

/// BOM upload response
#[derive(Debug, Serialize)]
pub struct BomUploadResponse {
    pub upload_id: Uuid,
    pub filename: String,
    pub format: String,
    pub total_rows: usize,
    pub suppliers: BomSupplierSummary,
    pub validation: BomValidationSummary,
    pub warnings: Vec<String>,
}

/// Supplier extraction summary
#[derive(Debug, Serialize)]
pub struct BomSupplierSummary {
    pub total: usize,
    pub complete: usize,
    pub incomplete: usize,
    pub duplicates_merged: usize,
}

/// Validation summary
#[derive(Debug, Serialize)]
pub struct BomValidationSummary {
    pub is_valid: bool,
    pub errors: usize,
    pub warnings: usize,
}

/// Upload and process BOM file
/// 
/// POST /api/v1/bom/upload
pub async fn upload_bom(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<BomUploadResponse>, (StatusCode, String)> {
    // Get file from multipart
    let field = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read upload: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?;
    
    let filename = field.file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown.csv".to_string());
    
    let data = field.bytes().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to read file data: {}", e)))?;
    
    // Parse BOM
    let parser = BomParser::new();
    let parsed_bom = parser.parse_bytes(&filename, &data, None)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Failed to parse BOM: {}", e)))?;
    
    // Validate
    let validator = BomValidator::new();
    let validation = validator.validate(&parsed_bom);
    
    // Extract suppliers
    let extractor = SupplierExtractor::new();
    let extraction = extractor.extract(&parsed_bom);
    
    // Combine warnings
    let mut all_warnings = parsed_bom.parse_warnings.clone();
    all_warnings.extend(extraction.warnings.clone());
    
    let format = match parsed_bom.format {
        elementa_utils::bom::BomFormat::Csv => "CSV",
        elementa_utils::bom::BomFormat::Excel => "Excel",
        elementa_utils::bom::BomFormat::Xml => "XML",
    };
    
    Ok(Json(BomUploadResponse {
        upload_id: parsed_bom.id,
        filename,
        format: format.to_string(),
        total_rows: parsed_bom.total_rows,
        suppliers: BomSupplierSummary {
            total: extraction.suppliers.len(),
            complete: extraction.complete_count,
            incomplete: extraction.incomplete_count,
            duplicates_merged: extraction.duplicate_count,
        },
        validation: BomValidationSummary {
            is_valid: validation.is_valid,
            errors: validation.error_count,
            warnings: validation.warning_count,
        },
        warnings: all_warnings,
    }))
}

/// Get extracted suppliers from a previous upload
/// 
/// GET /api/v1/bom/{upload_id}/suppliers
#[derive(Debug, Serialize)]
pub struct ExtractedSupplierResponse {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub contact_person: Option<String>,
    pub component_count: usize,
    pub is_complete: bool,
    pub missing_fields: Vec<String>,
}

pub async fn get_bom_suppliers(
    State(_state): State<AppState>,
    axum::extract::Path(upload_id): axum::extract::Path<Uuid>,
) -> Result<Json<Vec<ExtractedSupplierResponse>>, (StatusCode, String)> {
    // TODO: Retrieve from storage (for now, return not found)
    Err((StatusCode::NOT_FOUND, format!("BOM upload {} not found", upload_id)))
}
