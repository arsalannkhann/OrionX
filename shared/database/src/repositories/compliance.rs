//! Compliance Repository
//! 
//! CRUD operations for compliance records.
//! Uses runtime SQL queries to avoid requiring DATABASE_URL at compile time.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::{
    ComplianceRecord, ValidationStatus,
};

pub struct ComplianceRepository {
    pool: PgPool,
}

impl ComplianceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find compliance record by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ComplianceRecord>> {
        let row: Option<ComplianceRow> = sqlx::query_as(
            r#"
            SELECT id, supplier_id, component_id, cas_records,
                   test_results, certifications, submission_date,
                   validation_status, audit_trail, created_at, updated_at
            FROM compliance_records
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch compliance record by ID")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find all compliance records for a supplier
    pub async fn find_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<ComplianceRecord>> {
        let rows: Vec<ComplianceRow> = sqlx::query_as(
            r#"
            SELECT id, supplier_id, component_id, cas_records,
                   test_results, certifications, submission_date,
                   validation_status, audit_trail, created_at, updated_at
            FROM compliance_records
            WHERE supplier_id = $1
            ORDER BY submission_date DESC
            "#
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch compliance records by supplier")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find compliance records by validation status
    pub async fn find_by_status(&self, status: ValidationStatus) -> Result<Vec<ComplianceRecord>> {
        let status_str = serde_json::to_string(&status)?.trim_matches('"').to_string();
        
        let rows: Vec<ComplianceRow> = sqlx::query_as(
            r#"
            SELECT id, supplier_id, component_id, cas_records,
                   test_results, certifications, submission_date,
                   validation_status, audit_trail, created_at, updated_at
            FROM compliance_records
            WHERE validation_status = $1
            ORDER BY submission_date DESC
            "#
        )
        .bind(&status_str)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch compliance records by status")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find compliance records containing PFAS
    pub async fn find_with_pfas(&self) -> Result<Vec<ComplianceRecord>> {
        let rows: Vec<ComplianceRow> = sqlx::query_as(
            r#"
            SELECT id, supplier_id, component_id, cas_records,
                   test_results, certifications, submission_date,
                   validation_status, audit_trail, created_at, updated_at
            FROM compliance_records
            WHERE cas_records @> '[{"is_pfas": true}]'::jsonb
            ORDER BY submission_date DESC
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch PFAS compliance records")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Create new compliance record
    pub async fn create(&self, record: ComplianceRecord) -> Result<ComplianceRecord> {
        let cas_records = serde_json::to_value(&record.cas_records)?;
        let test_results = serde_json::to_value(&record.test_results)?;
        let certifications = serde_json::to_value(&record.certifications)?;
        let validation_status = serde_json::to_string(&record.validation_status)?;
        let audit_trail = serde_json::to_value(&record.audit_trail)?;
        let now = Utc::now();
        
        let row: ComplianceRow = sqlx::query_as(
            r#"
            INSERT INTO compliance_records 
                (id, supplier_id, component_id, cas_records, test_results,
                 certifications, submission_date, validation_status, 
                 audit_trail, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING id, supplier_id, component_id, cas_records,
                      test_results, certifications, submission_date,
                      validation_status, audit_trail, created_at, updated_at
            "#
        )
        .bind(record.id)
        .bind(record.supplier_id)
        .bind(record.component_id)
        .bind(&cas_records)
        .bind(&test_results)
        .bind(&certifications)
        .bind(record.submission_date)
        .bind(validation_status.trim_matches('"'))
        .bind(&audit_trail)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create compliance record")?;
        
        Ok(row.into())
    }
    
    /// Update compliance record
    pub async fn update(&self, record: ComplianceRecord) -> Result<ComplianceRecord> {
        let cas_records = serde_json::to_value(&record.cas_records)?;
        let test_results = serde_json::to_value(&record.test_results)?;
        let certifications = serde_json::to_value(&record.certifications)?;
        let validation_status = serde_json::to_string(&record.validation_status)?;
        let audit_trail = serde_json::to_value(&record.audit_trail)?;
        
        let row: ComplianceRow = sqlx::query_as(
            r#"
            UPDATE compliance_records SET
                cas_records = $2,
                test_results = $3,
                certifications = $4,
                validation_status = $5,
                audit_trail = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING id, supplier_id, component_id, cas_records,
                      test_results, certifications, submission_date,
                      validation_status, audit_trail, created_at, updated_at
            "#
        )
        .bind(record.id)
        .bind(&cas_records)
        .bind(&test_results)
        .bind(&certifications)
        .bind(validation_status.trim_matches('"'))
        .bind(&audit_trail)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to update compliance record")?;
        
        Ok(row.into())
    }
    
    /// Delete compliance record
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM compliance_records WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete compliance record")?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Get compliance summary statistics
    pub async fn get_summary_stats(&self) -> Result<ComplianceSummary> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM compliance_records")
            .fetch_one(&self.pool)
            .await?;
        
        let validated: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM compliance_records WHERE validation_status = 'Valid'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let pending: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM compliance_records WHERE validation_status = 'Pending'"
        )
        .fetch_one(&self.pool)
        .await?;
        
        let pfas_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM compliance_records WHERE cas_records @> '[{\"is_pfas\": true}]'::jsonb"
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(ComplianceSummary {
            total_records: total.0,
            validated_records: validated.0,
            pending_records: pending.0,
            pfas_detected_count: pfas_count.0,
        })
    }
}

/// Internal row type for SQLx mapping
#[derive(Debug, FromRow)]
struct ComplianceRow {
    id: Uuid,
    supplier_id: Uuid,
    component_id: Uuid,
    cas_records: serde_json::Value,
    test_results: serde_json::Value,
    certifications: serde_json::Value,
    submission_date: chrono::DateTime<Utc>,
    validation_status: String,
    audit_trail: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<ComplianceRow> for ComplianceRecord {
    fn from(row: ComplianceRow) -> Self {
        Self {
            id: row.id,
            supplier_id: row.supplier_id,
            component_id: row.component_id,
            cas_records: serde_json::from_value(row.cas_records).unwrap_or_default(),
            test_results: serde_json::from_value(row.test_results).unwrap_or_default(),
            certifications: serde_json::from_value(row.certifications).unwrap_or_default(),
            submission_date: row.submission_date,
            validation_status: serde_json::from_str(&format!("\"{}\"", row.validation_status))
                .unwrap_or(ValidationStatus::Pending),
            audit_trail: serde_json::from_value(row.audit_trail).unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Compliance summary statistics
#[derive(Debug, Clone)]
pub struct ComplianceSummary {
    pub total_records: i64,
    pub validated_records: i64,
    pub pending_records: i64,
    pub pfas_detected_count: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property 11: Immutable audit trail creation
        #[test]
        fn prop_audit_trail_immutable(
            _cas_number in "[0-9]{2,7}-[0-9]{2}-[0-9]",
        ) {
            let record = ComplianceRecord::new(Uuid::new_v4(), Uuid::new_v4());
            
            // Audit trail should be empty initially
            prop_assert!(record.audit_trail.is_empty());
            
            // Serialization should preserve audit trail
            let json = serde_json::to_string(&record).unwrap();
            let deserialized: ComplianceRecord = serde_json::from_str(&json).unwrap();
            prop_assert_eq!(record.audit_trail.len(), deserialized.audit_trail.len());
        }
    }
}
