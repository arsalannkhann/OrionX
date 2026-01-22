use anyhow::Result;
use redis::{aio::ConnectionManager, Client};

pub type RedisPool = ConnectionManager;

pub async fn create_redis_pool(redis_url: &str, _max_connections: u32) -> Result<RedisPool> {
    let client = Client::open(redis_url)?;
    let connection_manager = ConnectionManager::new(client).await?;
    
    tracing::info!("Connected to Redis cache");
    Ok(connection_manager)
}

pub async fn health_check(pool: &mut RedisPool) -> Result<()> {
    let _: String = redis::cmd("PING")
        .query_async(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Redis health check failed: {}", e))?;
    Ok(())
}