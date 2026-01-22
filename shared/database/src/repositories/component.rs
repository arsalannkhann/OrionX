//! Component Repository  
//!
//! CRUD operations for component records.

use anyhow::{Context, Result};
use chrono::Utc;
use sqlx::{PgPool, FromRow};
use uuid::Uuid;

use elementa_models::Component;

pub struct ComponentRepository {
    pool: PgPool,
}

impl ComponentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    /// Find component by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Component>> {
        let row: Option<ComponentRow> = sqlx::query_as(
            r#"
            SELECT id, part_number, description, cas_numbers, material_type,
                   supplier_id, specifications, created_at, updated_at
            FROM components
            WHERE id = $1
            "#
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch component by ID")?;
        
        Ok(row.map(|r| r.into()))
    }
    
    /// Find all components
    pub async fn find_all(&self) -> Result<Vec<Component>> {
        let rows: Vec<ComponentRow> = sqlx::query_as(
            r#"
            SELECT id, part_number, description, cas_numbers, material_type,
                   supplier_id, specifications, created_at, updated_at
            FROM components
            ORDER BY part_number
            "#
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch all components")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Find components by supplier
    pub async fn find_by_supplier(&self, supplier_id: Uuid) -> Result<Vec<Component>> {
        let rows: Vec<ComponentRow> = sqlx::query_as(
            r#"
            SELECT id, part_number, description, cas_numbers, material_type,
                   supplier_id, specifications, created_at, updated_at
            FROM components
            WHERE supplier_id = $1
            ORDER BY part_number
            "#
        )
        .bind(supplier_id)
        .fetch_all(&self.pool)
        .await
        .context("Failed to fetch components by supplier")?;
        
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }
    
    /// Create new component
    pub async fn create(&self, component: Component) -> Result<Component> {
        let cas_numbers = serde_json::to_value(&component.cas_numbers)?;
        let material_type = serde_json::to_string(&component.material_type)?;
        let specifications = serde_json::to_value(&component.specifications)?;
        let now = Utc::now();
        
        let row: ComponentRow = sqlx::query_as(
            r#"
            INSERT INTO components 
                (id, part_number, description, cas_numbers, material_type,
                 supplier_id, specifications, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, part_number, description, cas_numbers, material_type,
                      supplier_id, specifications, created_at, updated_at
            "#
        )
        .bind(component.id)
        .bind(&component.part_number)
        .bind(&component.description)
        .bind(&cas_numbers)
        .bind(material_type.trim_matches('"'))
        .bind(component.supplier_id)
        .bind(&specifications)
        .bind(now)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .context("Failed to create component")?;
        
        Ok(row.into())
    }
    
    /// Update component
    pub async fn update(&self, component: Component) -> Result<Component> {
        let cas_numbers = serde_json::to_value(&component.cas_numbers)?;
        let material_type = serde_json::to_string(&component.material_type)?;
        let specifications = serde_json::to_value(&component.specifications)?;
        
        let row: ComponentRow = sqlx::query_as(
            r#"
            UPDATE components SET
                part_number = $2,
                description = $3,
                cas_numbers = $4,
                material_type = $5,
                specifications = $6,
                updated_at = $7
            WHERE id = $1
            RETURNING id, part_number, description, cas_numbers, material_type,
                      supplier_id, specifications, created_at, updated_at
            "#
        )
        .bind(component.id)
        .bind(&component.part_number)
        .bind(&component.description)
        .bind(&cas_numbers)
        .bind(material_type.trim_matches('"'))
        .bind(&specifications)
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await
        .context("Failed to update component")?;
        
        Ok(row.into())
    }
    
    /// Delete component
    pub async fn delete(&self, id: Uuid) -> Result<bool> {
        let result = sqlx::query("DELETE FROM components WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete component")?;
        
        Ok(result.rows_affected() > 0)
    }
}

#[derive(Debug, FromRow)]
struct ComponentRow {
    id: Uuid,
    part_number: String,
    description: String,
    cas_numbers: serde_json::Value,
    material_type: String,
    supplier_id: Uuid,
    specifications: serde_json::Value,
    created_at: chrono::DateTime<Utc>,
    updated_at: chrono::DateTime<Utc>,
}

impl From<ComponentRow> for Component {
    fn from(row: ComponentRow) -> Self {
        use elementa_models::MaterialType;
        
        Self {
            id: row.id,
            part_number: row.part_number,
            description: row.description,
            cas_numbers: serde_json::from_value(row.cas_numbers).unwrap_or_default(),
            material_type: serde_json::from_str(&format!("\"{}\"", row.material_type))
                .unwrap_or(MaterialType::Other("Unknown".to_string())),
            supplier_id: row.supplier_id,
            specifications: serde_json::from_value(row.specifications)
                .unwrap_or_default(),
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
