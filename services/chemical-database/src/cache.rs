//! Chemical Cache
//! 
//! Redis-based caching for chemical lookups.

use anyhow::Result;

use redis::Client;
use redis::AsyncCommands;

/// Chemical cache configuration
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CacheConfig {
    pub redis_url: String,
    pub ttl_seconds: usize,
    pub prefix: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: "redis://localhost:6379".to_string(),
            ttl_seconds: 86400, // 24 hours
            prefix: "elementa:chemical:".to_string(),
        }
    }
}

/// Chemical cache using Redis
#[allow(dead_code)]
pub struct ChemicalCache {
    client: Client,
    ttl_seconds: usize,
    config: CacheConfig,
}

#[allow(dead_code)]
impl ChemicalCache {
    pub fn new(config: CacheConfig) -> Self {
        let client = Client::open(config.redis_url.clone())
            .expect("Failed to create Redis client");
        
        Self {
            client,
            ttl_seconds: config.ttl_seconds,
            config,
        }
    }
    
    /// Get chemical from cache
    pub async fn get(&self, cas_number: &str) -> Result<Option<String>> {
        let key = format!("{}{}", self.config.prefix, cas_number);
        
        let mut con = self.client.get_async_connection().await?;
        let result: Option<String> = con.get(key).await?;
        Ok(result)
    }
    
    /// Set chemical in cache
    pub async fn set(&self, cas_number: &str, data: &str) -> Result<()> {
        let key = format!("{}{}", self.config.prefix, cas_number);
        
        let mut con = self.client.get_async_connection().await?;
        // Set with expiration (EX)
        let _: () = redis::cmd("SET")
            .arg(key)
            .arg(data)
            .arg("EX")
            .arg(self.ttl_seconds)
            .query_async(&mut con)
            .await?;
        Ok(())
    }
    
    /// Invalidate cache entry
    pub async fn invalidate(&self, cas_number: &str) -> Result<()> {
        let key = format!("{}{}", self.config.prefix, cas_number);
        
        let mut con = self.client.get_async_connection().await?;
        let _: () = con.del(key).await?;
        Ok(())
    }
    
    /// Clear all chemical cache entries
    pub async fn clear(&self) -> Result<usize> {
        let mut con = self.client.get_async_connection().await?;
        let _: () = redis::cmd("FLUSHDB").query_async(&mut con).await?;
        Ok(0)
    }
}

impl Default for ChemicalCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}
