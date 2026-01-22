//! Elementa Chemical Database Service
//! 
//! CAS number validation and PFAS classification service.
//! Integrates with EPA databases and CAS Registry.

use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;

mod service;
mod epa_client;
mod cache;

use service::ChemicalService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    info!("Starting Elementa Chemical Database Service");
    
    // Initialize service
    let service = ChemicalService::new();
    
    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/chemicals/:cas_number", get(get_chemical))
        .route("/api/v1/chemicals/:cas_number/validate", get(validate_cas))
        .route("/api/v1/chemicals/:cas_number/pfas", get(classify_pfas))
        .route("/api/v1/chemicals/batch", post(batch_lookup))
        .route("/api/v1/pfas/list", get(get_pfas_list))
        .route("/api/v1/pfas/sync", post(sync_pfas_database))
        .layer(TraceLayer::new_for_http())
        .with_state(service);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8082));
    let listener = TcpListener::bind(&addr).await?;
    info!("Chemical Database Service listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "chemical-database",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Get chemical by CAS number
#[derive(Debug, Serialize)]
struct ChemicalResponse {
    cas_number: String,
    chemical_name: String,
    molecular_formula: Option<String>,
    molecular_weight: Option<f64>,
    is_pfas: bool,
    pfas_classification: Option<PfasClassificationResponse>,
    regulatory_status: Vec<RegulatoryStatusResponse>,
}

#[derive(Debug, Serialize)]
struct PfasClassificationResponse {
    is_pfas: bool,
    confidence: f64,
    classification_source: String,
    lists: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RegulatoryStatusResponse {
    regulation: String,
    status: String,
    reporting_threshold: Option<String>,
}

async fn get_chemical(
    State(service): State<ChemicalService>,
    Path(cas_number): Path<String>,
) -> Result<Json<ChemicalResponse>, (StatusCode, String)> {
    let chemical = service.lookup(&cas_number).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, format!("Chemical {} not found", cas_number)))?;
    
    Ok(Json(ChemicalResponse {
        cas_number: chemical.cas_number,
        chemical_name: chemical.chemical_name,
        molecular_formula: chemical.molecular_formula,
        molecular_weight: chemical.molecular_weight,
        is_pfas: chemical.is_pfas,
        pfas_classification: chemical.pfas_classification.map(|c| PfasClassificationResponse {
            is_pfas: c.is_pfas,
            confidence: c.confidence,
            classification_source: c.source.clone(),
            lists: c.regulatory_lists.iter().map(|l| l.list_name.clone()).collect(),
        }),
        regulatory_status: chemical.regulatory_status.iter().map(|s| RegulatoryStatusResponse {
            regulation: s.source.clone(),
            status: s.status.clone(),
            reporting_threshold: s.reporting_threshold.clone(),
        }).collect(),
    }))
}

/// Validate CAS number format and checksum
#[derive(Debug, Serialize)]
struct CasValidationResponse {
    cas_number: String,
    is_valid: bool,
    format_valid: bool,
    checksum_valid: bool,
    normalized: String,
    errors: Vec<String>,
}

async fn validate_cas(
    State(service): State<ChemicalService>,
    Path(cas_number): Path<String>,
) -> Json<CasValidationResponse> {
    let validation = service.validate_cas(&cas_number);
    
    Json(CasValidationResponse {
        cas_number: cas_number.clone(),
        is_valid: validation.is_valid,
        format_valid: validation.format_valid,
        checksum_valid: validation.checksum_valid,
        normalized: validation.normalized,
        errors: validation.errors,
    })
}

/// Classify CAS number for PFAS status
#[derive(Debug, Serialize)]
struct PfasResponse {
    cas_number: String,
    is_pfas: bool,
    confidence: f64,
    source: String,
    regulatory_lists: Vec<String>,
    reporting_requirements: Vec<String>,
}

async fn classify_pfas(
    State(service): State<ChemicalService>,
    Path(cas_number): Path<String>,
) -> Result<Json<PfasResponse>, (StatusCode, String)> {
    let classification = service.classify_pfas(&cas_number).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(PfasResponse {
        cas_number,
        is_pfas: classification.is_pfas,
        confidence: classification.confidence,
        source: classification.source,
        regulatory_lists: classification.regulatory_lists.iter().map(|l| l.list_name.clone()).collect(),
        reporting_requirements: classification.reporting_requirements.iter().map(|r| r.description.clone()).collect(),
    }))
}

/// Batch lookup multiple CAS numbers
#[derive(Debug, Deserialize)]
struct BatchLookupRequest {
    cas_numbers: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BatchLookupResponse {
    results: Vec<BatchLookupResult>,
    found: usize,
    not_found: usize,
    pfas_count: usize,
}

#[derive(Debug, Serialize)]
struct BatchLookupResult {
    cas_number: String,
    found: bool,
    chemical_name: Option<String>,
    is_pfas: Option<bool>,
    error: Option<String>,
}

async fn batch_lookup(
    State(service): State<ChemicalService>,
    Json(request): Json<BatchLookupRequest>,
) -> Json<BatchLookupResponse> {
    let mut results = Vec::new();
    let mut found = 0;
    let mut not_found = 0;
    let mut pfas_count = 0;
    
    for cas in request.cas_numbers {
        match service.lookup(&cas).await {
            Ok(Some(chemical)) => {
                found += 1;
                if chemical.is_pfas {
                    pfas_count += 1;
                }
                results.push(BatchLookupResult {
                    cas_number: cas,
                    found: true,
                    chemical_name: Some(chemical.chemical_name),
                    is_pfas: Some(chemical.is_pfas),
                    error: None,
                });
            }
            Ok(None) => {
                not_found += 1;
                results.push(BatchLookupResult {
                    cas_number: cas,
                    found: false,
                    chemical_name: None,
                    is_pfas: None,
                    error: Some("Not found".to_string()),
                });
            }
            Err(e) => {
                not_found += 1;
                results.push(BatchLookupResult {
                    cas_number: cas,
                    found: false,
                    chemical_name: None,
                    is_pfas: None,
                    error: Some(e.to_string()),
                });
            }
        }
    }
    
    Json(BatchLookupResponse {
        results,
        found,
        not_found,
        pfas_count,
    })
}

/// Get PFAS list statistics
#[derive(Debug, Serialize)]
struct PfasListResponse {
    total_substances: usize,
    sources: Vec<PfasSourceInfo>,
    last_updated: String,
}

#[derive(Debug, Serialize)]
struct PfasSourceInfo {
    name: String,
    count: usize,
    last_updated: String,
}

async fn get_pfas_list(
    State(service): State<ChemicalService>,
) -> Json<PfasListResponse> {
    let stats = service.get_pfas_stats().await;
    
    Json(PfasListResponse {
        total_substances: stats.total,
        sources: stats.sources.into_iter().map(|s| PfasSourceInfo {
            name: s.name,
            count: s.count,
            last_updated: s.last_updated,
        }).collect(),
        last_updated: stats.last_sync,
    })
}

/// Sync PFAS database from external sources
#[derive(Debug, Serialize)]
struct SyncResponse {
    success: bool,
    new_substances: usize,
    updated_substances: usize,
    errors: Vec<String>,
}

async fn sync_pfas_database(
    State(service): State<ChemicalService>,
) -> Result<Json<SyncResponse>, (StatusCode, String)> {
    let result = service.sync_from_sources().await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(SyncResponse {
        success: result.errors.is_empty(),
        new_substances: result.new_count,
        updated_substances: result.updated_count,
        errors: result.errors,
    }))
}