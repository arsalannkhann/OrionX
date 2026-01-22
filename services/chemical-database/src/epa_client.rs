//! EPA API Client
//! 
//! Client for EPA chemical databases (PFAS Master List, TSCA Inventory).

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// EPA API client
#[allow(dead_code)]
pub struct EpaClient {
    client: Client,
    base_url: String,
}

#[allow(dead_code)]
impl EpaClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            base_url: "https://comptox.epa.gov/dashboard".to_string(),
        }
    }
    
    /// Lookup chemical in EPA CompTox database
    pub async fn lookup_chemical(&self, cas_number: &str) -> Result<Option<EpaChemical>> {
        // EPA CompTox API endpoint
        let url = format!("{}/dsstoxdb/results?search={}", self.base_url, cas_number);
        
        let response = self.client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to query EPA database")?;
        
        if !response.status().is_success() {
            return Ok(None);
        }
        
        // Parse response (simplified - actual EPA API has different structure)
        let data: ChemicalDetailsResponse = response.json().await
            .context("Failed to parse EPA response")?;
        
        Ok(data.chemicals.into_iter().next())
    }
    
    /// Get PFAS Master List substances
    pub async fn get_pfas_list(&self) -> Result<Vec<PfasSubstance>> {
        // EPA PFAS Master List endpoint
        // Note: Actual implementation would use proper EPA API
        let url = format!("{}/chemical-lists/PFASMASTER", self.base_url);
        
        let response = self.client.get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .context("Failed to query PFAS Master List")?;
        
        if !response.status().is_success() {
            return Ok(Vec::new());
        }
        
        let data: PfasListResponse = response.json().await
            .context("Failed to parse PFAS list")?;
        
        Ok(data.chemicals)
    }
}

/// EPA chemical search response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ChemicalDetailsResponse {
    pub chemicals: Vec<EpaChemical>,
}

/// EPA chemical data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct EpaChemical {
    pub dtxsid: String,
    pub cas_number: Option<String>,
    pub preferred_name: String,
    pub molecular_formula: Option<String>,
    pub molecular_weight: Option<f64>,
    pub is_pfas: bool,
}

/// PFAS list response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PfasListResponse {
    pub total_count: i32,
    pub chemicals: Vec<PfasSubstance>,
}

/// PFAS substance
#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(dead_code)]
pub struct PfasSubstance {
    pub cas_number: Option<String>,
    pub preferred_name: String,
    pub date_added: String,
}

impl Default for EpaClient {
    fn default() -> Self {
        Self::new()
    }
}
