use anyhow::Result;
use sqlx::{Pool, Postgres};
use std::time::Duration;

pub type PostgresPool = Pool<Postgres>;

pub async fn create_postgres_pool(database_url: &str, max_connections: u32) -> Result<PostgresPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;
    
    tracing::info!("Connected to PostgreSQL database");
    Ok(pool)
}

pub async fn health_check(pool: &PostgresPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .execute(pool)
        .await?;
    Ok(())
}