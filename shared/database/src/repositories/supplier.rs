//! Supplier Repository
//! 
//! CRUD operations for supplier records.
//! Uses runtime SQL queries (unchecked) to avoid requiring DATABASE_URL at compile time.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::{
    SupplierRecord, SupplierRelationship,
    ComplianceStatus, RiskLevel,
};

pub struct SupplierRepository {
    pool: PgPool,
}

impl SupplierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find supplier by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<SupplierRecord>> {
        let row: Option<SupplierRow> = sqlx::query_as(
            r#"
            SELECT id, name, contact_info, relationship, 
                   compliance_history, communication_preferences, 
                   risk_profile, created_at, updated_at
            FROM suppliers
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch supplier by ID")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find all suppliers
    pub async fn find_all(&self) -> Result<Vec<SupplierRecord>> {
        let rows: Vec<SupplierRow> = sqlx::query_as(
            r#"
            SELECT id, name, contact_info, relationship, 
                   compliance_history, communication_preferences, 
                   risk_profile, created_at, updated_at
            FROM suppliers
            ORDER BY name
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch all suppliers")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find suppliers by compliance status
    pub async fn find_by_compliance_status(&self, status: ComplianceStatus) -> Result<Vec<SupplierRecord>> {
        let status_str = serde_json::to_string(&status)?;
        let pattern = format!("[{{\"status\": {}}}]", status_str);
        
        let rows: Vec<SupplierRow> = sqlx::query_as(
            r#"
            SELECT id, name, contact_info, relationship, 
                   compliance_history, communication_preferences, 
                   risk_profile, created_at, updated_at
            FROM suppliers
            WHERE compliance_history @> $1::jsonb
            ORDER BY name
            "#
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch suppliers by compliance status")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find suppliers by risk level
    pub async fn find_by_risk_level(&self, risk: RiskLevel) -> Result<Vec<SupplierRecord>> {
        let risk_str = serde_json::to_string(&risk)?;
        let risk_value = risk_str.trim_matches('"');
        
        let rows: Vec<SupplierRow> = sqlx::query_as(
            r#"
            SELECT id, name, contact_info, relationship, 
                   compliance_history, communication_preferences, 
                   risk_profile, created_at, updated_at
            FROM suppliers
            WHERE risk_profile->>'compliance_risk' = $1
            ORDER BY name
            "#
        )
        .bind(risk_value)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch suppliers by risk level")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Create new supplier
    pub async fn create(&self, supplier: SupplierRecord) -> Result<SupplierRecord> {
        let contact_info = serde_json::to_value(&supplier.contact_info)?;
        let relationship = serde_json::to_string(&supplier.relationship)?;
        let compliance_history = serde_json::to_value(&supplier.compliance_history)?;
        let communication_preferences = serde_json::to_value(&supplier.communication_preferences)?;
        let risk_profile = serde_json::to_value(&supplier.risk_profile)?;
        let now = Utc::now();
        
        let row: SupplierRow = sqlx::query_as(
            r#"
            INSERT INTO suppliers 
                (id, name, contact_info, relationship, compliance_history, 
                 communication_preferences, risk_profile, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, name, contact_info, relationship, 
                      compliance_history, communication_preferences, 
                      risk_profile, created_at, updated_at
            "#
        )
        .bind(supplier.id)
        .bind(&supplier.name)
        .bind(&contact_info)
        .bind(relationship.trim_matches('"'))
        .bind(&compliance_history)
        .bind(&communication_preferences)
        .bind(&risk_profile)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create supplier")?;
        
        Ok(row.into())
    }
    
    /// Update existing supplier
    pub async fn update(&self, supplier: SupplierRecord) -> Result<SupplierRecord> {
        let contact_info = serde_json::to_value(&supplier.contact_info)?;
        let relationship = serde_json::to_string(&supplier.relationship)?;
        let compliance_history = serde_json::to_value(&supplier.compliance_history)?;
        let communication_preferences = serde_json::to_value(&supplier.communication_preferences)?;
        let risk_profile = serde_json::to_value(&supplier.risk_profile)?;
        
        let row: SupplierRow = sqlx::query_as(
            r#"
            UPDATE suppliers SET
                name = $2,
                contact_info = $3,
                relationship = $4,
                compliance_history = $5,
                communication_preferences = $6,
                risk_profile = $7,
                updated_at = $8
            WHERE id = $1
            RETURNING id, name, contact_info, relationship, 
                      compliance_history, communication_preferences, 
                      risk_profile, created_at, updated_at
            "#
        )
        .bind(supplier.id)
        .bind(&supplier.name)
        .bind(&contact_info)
        .bind(relationship.trim_matches('"'))
        .bind(&compliance_history)
        .bind(&communication_preferences)
        .bind(&risk_profile)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to update supplier")?;
        
        Ok(row.into())
    }
    
    /// Delete supplier by ID
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM suppliers WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete supplier")?;
        
        Ok(result.rows_affected() > 0)
    }
    
    /// Search suppliers by name
    pub async fn search_by_name(&self, query: &str) -> Result<Vec<SupplierRecord>> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        
        let rows: Vec<SupplierRow> = sqlx::query_as(
            r#"
            SELECT id, name, contact_info, relationship, 
                   compliance_history, communication_preferences, 
                   risk_profile, created_at, updated_at
            FROM suppliers
            WHERE LOWER(name) LIKE $1
            ORDER BY name
            LIMIT 100
            "#
        )
        .bind(&search_pattern)
        .fetch_all(&self.pool)
        .await
        .context("Failed to search suppliers by name")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Count total suppliers
    pub async fn count(&self) -> Result<i64> {
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM suppliers")
            .fetch_one(&self.pool)
            .await
            .context("Failed to count suppliers")?;
        
        Ok(row.0)
    }
}

/// Internal row type for SQLx mapping
#[derive(Debug, FromRow)]
struct SupplierRow {
    id: Uuid,
    name: String,
    contact_info: serde_json::Value,
    relationship: String,
    compliance_history: serde_json::Value,
    communication_preferences: serde_json::Value,
    risk_profile: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<SupplierRow> for SupplierRecord {
    fn from(row: SupplierRow) -> Self {
        Self {
            id: row.id,
            name: row.name,
            contact_info: serde_json::from_value(row.contact_info).unwrap_or_default(),
            relationship: serde_json::from_str(&format!("\"{}\"", row.relationship))
                .unwrap_or(SupplierRelationship::Standard),
            compliance_history: serde_json::from_value(row.compliance_history).unwrap_or_default(),
            communication_preferences: serde_json::from_value(row.communication_preferences)
                .unwrap_or_default(),
            risk_profile: serde_json::from_value(row.risk_profile).unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    proptest! {
        /// Property 1: Serialization round-trip consistency
        /// For any valid SupplierRecord, serializing and deserializing should produce equal data
        #[test]
        fn prop_serialization_roundtrip(
            name in "[a-zA-Z ]{1,50}",
            email in "[a-z]{5,10}@[a-z]{5,10}\\.[a-z]{2,3}",
        ) {
            let supplier = SupplierRecord {
                id: Uuid::new_v4(),
                name,
                contact_info: ContactInfo {
                    primary_email: email,
                    ..Default::default()
                },
                ..Default::default()
            };
            
            let json = serde_json::to_string(&supplier).unwrap();
            let deserialized: SupplierRecord = serde_json::from_str(&json).unwrap();
            
            prop_assert_eq!(supplier.id, deserialized.id);
            prop_assert_eq!(supplier.name, deserialized.name);
            prop_assert_eq!(
                supplier.contact_info.primary_email,
                deserialized.contact_info.primary_email
            );
        }
    }
}
