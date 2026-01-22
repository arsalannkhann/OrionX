//! Elementa Document Processing Service
//! 
//! VLM-powered document extraction for compliance data.
//! Supports PDF, images, and scanned documents.

use anyhow::Result;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

mod vlm_client;
mod pdf_processor;
mod extraction;

use extraction::{
    DocumentExtractor, CasExtractionResponse, TestResultResponse, 
    CertificationResponse, UncertaintyResponse
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting Elementa Document Processing Service");
    
    let extractor = DocumentExtractor::new();
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/documents/upload", post(upload_document))
        .route("/api/v1/documents/:id", get(get_document))
        .route("/api/v1/documents/:id/extract", post(extract_data))
        .route("/api/v1/documents/:id/cas-numbers", get(get_cas_numbers))
        .layer(TraceLayer::new_for_http())
        .with_state(extractor);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8083));
    let listener = TcpListener::bind(&addr).await?;
    info!("Document Processing Service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "document-processing",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Document upload response
#[derive(Debug, Serialize)]
pub struct DocumentUploadResponse {
    pub document_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub size_bytes: usize,
    pub status: String,
}

/// Upload compliance document
async fn upload_document(
    State(extractor): State<DocumentExtractor>,
    mut multipart: Multipart,
) -> Result<Json<DocumentUploadResponse>, (StatusCode, String)> {
    let field = multipart.next_field().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Upload error: {}", e)))?
        .ok_or((StatusCode::BAD_REQUEST, "No file provided".to_string()))?;
    
    let filename = field.file_name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    
    let content_type = field.content_type()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());
    
    let data = field.bytes().await
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Read error: {}", e)))?;
    
    let doc_id = extractor.store_document(&filename, &content_type, &data).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(DocumentUploadResponse {
        document_id: doc_id,
        filename,
        file_type: content_type,
        size_bytes: data.len(),
        status: "uploaded".to_string(),
    }))
}

/// Get document metadata
#[derive(Debug, Serialize)]
pub struct DocumentResponse {
    pub document_id: Uuid,
    pub filename: String,
    pub file_type: String,
    pub upload_date: String,
    pub processing_status: String,
    pub extraction_result: Option<ExtractionResultResponse>,
}

async fn get_document(
    State(extractor): State<DocumentExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Json<DocumentResponse>, (StatusCode, String)> {
    let doc = extractor.get_document(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;
    
    Ok(Json(DocumentResponse {
        document_id: doc.id,
        filename: doc.filename,
        file_type: doc.file_type,
        upload_date: doc.upload_date,
        processing_status: doc.status,
        extraction_result: doc.extraction.map(|e| ExtractionResultResponse {
            cas_numbers: e.cas_numbers,
            test_results: e.test_results,
            certifications: e.certifications,
            confidence: e.overall_confidence,
            uncertainties: e.uncertainties,
        }),
    }))
}

/// Extraction result response
#[derive(Debug, Serialize)]
pub struct ExtractionResultResponse {
    pub cas_numbers: Vec<CasExtractionResponse>,
    pub test_results: Vec<TestResultResponse>,
    pub certifications: Vec<CertificationResponse>,
    pub confidence: f64,
    pub uncertainties: Vec<UncertaintyResponse>,
}

/// Trigger extraction for a document
#[derive(Debug, Serialize)]
pub struct ExtractResponse {
    pub document_id: Uuid,
    pub status: String,
    pub cas_numbers_found: usize,
    pub test_results_found: usize,
    pub overall_confidence: f64,
    pub needs_review: bool,
}

async fn extract_data(
    State(extractor): State<DocumentExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Json<ExtractResponse>, (StatusCode, String)> {
    let result = extractor.extract(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(ExtractResponse {
        document_id: id,
        status: "extracted".to_string(),
        cas_numbers_found: result.cas_numbers.len(),
        test_results_found: result.test_results.len(),
        overall_confidence: result.overall_confidence,
        needs_review: result.overall_confidence < 0.7 || !result.uncertainties.is_empty(),
    }))
}

/// Get extracted CAS numbers from document
async fn get_cas_numbers(
    State(extractor): State<DocumentExtractor>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CasExtractionResponse>>, (StatusCode, String)> {
    let doc = extractor.get_document(id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Document not found".to_string()))?;
    
    let cas_numbers = doc.extraction
        .map(|e| e.cas_numbers)
        .unwrap_or_default();
    
    Ok(Json(cas_numbers))
}