use anyhow::Result;
use sqlx::PgPool;

pub async fn run_postgres_migrations(pool: &PgPool) -> Result<()> {
    tracing::info!("Running PostgreSQL migrations");
    
    // Create suppliers table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS suppliers (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            name VARCHAR NOT NULL,
            contact_info JSONB NOT NULL,
            relationship VARCHAR NOT NULL,
            compliance_history JSONB NOT NULL DEFAULT '[]',
            communication_preferences JSONB NOT NULL,
            risk_profile JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create components table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS components (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            part_number VARCHAR NOT NULL,
            description TEXT NOT NULL,
            cas_numbers JSONB NOT NULL DEFAULT '[]',
            material_type VARCHAR NOT NULL,
            supplier_id UUID NOT NULL REFERENCES suppliers(id),
            specifications JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create compliance_records table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS compliance_records (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            supplier_id UUID NOT NULL REFERENCES suppliers(id),
            component_id UUID NOT NULL REFERENCES components(id),
            cas_records JSONB NOT NULL DEFAULT '[]',
            test_results JSONB NOT NULL DEFAULT '[]',
            certifications JSONB NOT NULL DEFAULT '[]',
            submission_date TIMESTAMPTZ NOT NULL,
            validation_status VARCHAR NOT NULL,
            audit_trail JSONB NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create chemical_substances table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS chemical_substances (
            cas_number VARCHAR PRIMARY KEY,
            chemical_name VARCHAR NOT NULL,
            molecular_formula VARCHAR,
            molecular_weight DECIMAL,
            is_pfas BOOLEAN NOT NULL DEFAULT FALSE,
            pfas_classification JSONB,
            regulatory_status JSONB NOT NULL,
            last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create workflows table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workflows (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            client_id UUID NOT NULL,
            campaign_name VARCHAR NOT NULL,
            suppliers JSONB NOT NULL DEFAULT '[]',
            status VARCHAR NOT NULL,
            start_date TIMESTAMPTZ NOT NULL,
            deadline TIMESTAMPTZ NOT NULL,
            progress JSONB NOT NULL,
            escalations JSONB NOT NULL DEFAULT '[]',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create agent_tasks table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS agent_tasks (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            workflow_id UUID NOT NULL REFERENCES workflows(id),
            task_type VARCHAR NOT NULL,
            supplier_id UUID NOT NULL REFERENCES suppliers(id),
            context JSONB NOT NULL,
            status VARCHAR NOT NULL,
            retry_count INTEGER NOT NULL DEFAULT 0,
            max_retries INTEGER NOT NULL DEFAULT 3,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            completed_at TIMESTAMPTZ
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create email_communications table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS email_communications (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            thread_id VARCHAR NOT NULL,
            supplier_id UUID NOT NULL REFERENCES suppliers(id),
            direction VARCHAR NOT NULL,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            attachments JSONB NOT NULL DEFAULT '[]',
            sent_at TIMESTAMPTZ,
            received_at TIMESTAMPTZ,
            delivery_status VARCHAR NOT NULL,
            processing_status VARCHAR NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create audit_entries table
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS audit_entries (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            timestamp TIMESTAMPTZ NOT NULL,
            action VARCHAR NOT NULL,
            user_id UUID,
            agent_id VARCHAR,
            details JSONB NOT NULL,
            source_document JSONB,
            hash VARCHAR NOT NULL,
            previous_hash VARCHAR,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Create indexes for better performance
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_suppliers_name ON suppliers(name)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_components_supplier_id ON components(supplier_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_compliance_records_supplier_id ON compliance_records(supplier_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_email_communications_supplier_id ON email_communications(supplier_id)")
        .execute(pool)
        .await?;
    
    sqlx::query("CREATE INDEX IF NOT EXISTS idx_audit_entries_timestamp ON audit_entries(timestamp)")
        .execute(pool)
        .await?;

    tracing::info!("PostgreSQL migrations completed successfully");
    Ok(())
}