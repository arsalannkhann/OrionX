//! Chemical Repository
//!
//! CRUD operations for chemical substance records.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};


use elementa_models::ChemicalSubstance;

pub struct ChemicalRepository {
    pool: PgPool,
}

impl ChemicalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find chemical by CAS number
    pub async fn find_by_cas(&self, cas_number: &str) -> Result<Option<ChemicalSubstance>> {
        let row: Option<ChemicalRow> = sqlx::query_as(
            r#"
            SELECT cas_number, chemical_name, molecular_formula, molecular_weight, is_pfas,
                   pfas_classification, regulatory_status, last_updated
            FROM chemicals
            WHERE cas_number = $1
            "#
        )
        .bind(cas_number)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch chemical by CAS")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find all PFAS substances
    pub async fn find_all_pfas(&self) -> Result<Vec<ChemicalSubstance>> {
        let rows: Vec<ChemicalRow> = sqlx::query_as(
            r#"
            SELECT cas_number, chemical_name, molecular_formula, molecular_weight, is_pfas,
                   pfas_classification, regulatory_status, last_updated
            FROM chemicals
            WHERE is_pfas = true
            ORDER BY chemical_name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch PFAS chemicals")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Upsert chemical (insert or update)
    pub async fn upsert(&self, chemical: ChemicalSubstance) -> Result<ChemicalSubstance> {
        let pfas_classification = serde_json::to_value(&chemical.pfas_classification)?;
        let regulatory_status = serde_json::to_value(&chemical.regulatory_status)?;
        let now = Utc::now();
        
        let row: ChemicalRow = sqlx::query_as(
            r#"
            INSERT INTO chemicals 
                (cas_number, chemical_name, molecular_formula, molecular_weight, is_pfas,
                 pfas_classification, regulatory_status, last_updated)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (cas_number) DO UPDATE SET
                chemical_name = EXCLUDED.chemical_name,
                molecular_formula = EXCLUDED.molecular_formula,
                molecular_weight = EXCLUDED.molecular_weight,
                is_pfas = EXCLUDED.is_pfas,
                pfas_classification = EXCLUDED.pfas_classification,
                regulatory_status = EXCLUDED.regulatory_status,
                last_updated = EXCLUDED.last_updated
            RETURNING cas_number, chemical_name, molecular_formula, molecular_weight, is_pfas,
                      pfas_classification, regulatory_status, last_updated
            "#
        )
        .bind(&chemical.cas_number)
        .bind(&chemical.chemical_name)
        .bind(&chemical.molecular_formula)
        .bind(chemical.molecular_weight)
        .bind(chemical.is_pfas)
        .bind(&pfas_classification)
        .bind(&regulatory_status)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to upsert chemical")?;
        
        Ok(row.into())
    }
    
    /// Bulk upsert chemicals
    pub async fn bulk_upsert(&self, chemicals: Vec<ChemicalSubstance>) -> Result<usize> {
        let mut count = 0;
        for chemical in chemicals {
            self.upsert(chemical).await?;
            count += 1;
        }
        Ok(count)
    }
    
    /// Count PFAS substances
    pub async fn count_pfas(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM chemicals WHERE is_pfas = true")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count PFAS")?;
        
        Ok(row.0)
    }
}

#[derive(Debug, FromRow)]
struct ChemicalRow {
    cas_number: String,
    chemical_name: String,
    molecular_formula: Option<String>,
    molecular_weight: Option<f64>,
    is_pfas: bool,
    pfas_classification: serde_json::Value,
    regulatory_status: serde_json::Value,
    last_updated: chrono::DateTime<Utc>,
}

impl From<ChemicalRow> for ChemicalSubstance {
    fn from(row: ChemicalRow) -> Self {
        use elementa_models::chemical::RegulatoryStatus;
        
        Self {
            cas_number: row.cas_number,
            chemical_name: row.chemical_name,
            molecular_formula: row.molecular_formula,
            molecular_weight: row.molecular_weight,
            is_pfas: row.is_pfas,
            pfas_classification: serde_json::from_value(row.pfas_classification).ok(),
            regulatory_status: serde_json::from_value(row.regulatory_status)
                .unwrap_or_else(|_| RegulatoryStatus {
                    regulatory_lists: Vec::new(),
                    reporting_requirements: Vec::new(),
                    restrictions: Vec::new(),
                    last_updated: chrono::Utc::now(),
                }),
            last_updated: row.last_updated,
        }
    }
}
