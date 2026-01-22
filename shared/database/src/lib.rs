pub mod postgres;
pub mod mongodb;
pub mod redis;
pub mod migrations;
pub mod repositories;

pub use postgres::{PostgresPool, create_postgres_pool, health_check as postgres_health_check};
pub use mongodb::{MongoClient, MongoDatabase, create_mongo_client, get_database, health_check as mongo_health_check};
pub use redis::{RedisPool, create_redis_pool, health_check as redis_health_check};
pub use repositories::*;

use anyhow::Result;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub mongodb_url: String,
    pub redis_url: String,
    pub max_connections: u32,
    pub connection_timeout: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            postgres_url: "postgresql://elementa:elementa@localhost:5432/elementa".to_string(),
            mongodb_url: "mongodb://localhost:27017/elementa".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            max_connections: 10,
            connection_timeout: Duration::from_secs(30),
        }
    }
}

pub async fn initialize_databases(config: &DatabaseConfig) -> Result<(PostgresPool, MongoClient, RedisPool)> {
    let postgres_pool = create_postgres_pool(&config.postgres_url, config.max_connections).await?;
    let mongo_client = create_mongo_client(&config.mongodb_url).await?;
    let redis_pool = create_redis_pool(&config.redis_url, config.max_connections).await?;
    
    // Run migrations
    migrations::run_postgres_migrations(&postgres_pool).await?;
    
    Ok((postgres_pool, mongo_client, redis_pool))
}