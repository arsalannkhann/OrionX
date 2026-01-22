use anyhow::Result;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub email: EmailConfig,
    pub vlm: VLMConfig,
    pub chemical_db: ChemicalDbConfig,
    pub logging: LoggingConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_request_size: usize,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub postgres_url: String,
    pub mongodb_url: String,
    pub redis_url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub from_address: String,
    pub from_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLMConfig {
    pub api_url: String,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemicalDbConfig {
    pub epa_api_url: String,
    pub cas_api_url: String,
    pub api_keys: std::collections::HashMap<String, String>,
    pub cache_ttl_hours: u64,
    pub update_interval_hours: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file_path: Option<String>,
    pub max_file_size: Option<u64>,
    pub max_files: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub health_check_interval_seconds: u64,
    pub prometheus_namespace: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let config = Config::builder()
            // Start with default values
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific config
            .add_source(
                File::with_name(&format!(
                    "config/{}",
                    env::var("ENVIRONMENT").unwrap_or_else(|_| "development".into())
                ))
                .required(false),
            )
            // Add local config (gitignored)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with ELEMENTA prefix
            .add_source(Environment::with_prefix("ELEMENTA").separator("__"));

        config.build()?.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_request_size: 16 * 1024 * 1024, // 16MB
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                postgres_url: "postgresql://elementa:elementa@localhost:5432/elementa".to_string(),
                mongodb_url: "mongodb://localhost:27017/elementa".to_string(),
                redis_url: "redis://localhost:6379".to_string(),
                max_connections: 10,
                connection_timeout_seconds: 30,
            },
            email: EmailConfig {
                smtp_host: "localhost".to_string(),
                smtp_port: 587,
                smtp_username: "elementa".to_string(),
                smtp_password: "password".to_string(),
                imap_host: "localhost".to_string(),
                imap_port: 993,
                from_address: "noreply@elementa.com".to_string(),
                from_name: "Elementa Compliance System".to_string(),
            },
            vlm: VLMConfig {
                api_url: "https://api.openai.com/v1".to_string(),
                api_key: "your-api-key".to_string(),
                model: "gpt-4-vision-preview".to_string(),
                max_tokens: 4096,
                temperature: 0.1,
                timeout_seconds: 60,
            },
            chemical_db: ChemicalDbConfig {
                epa_api_url: "https://api.epa.gov".to_string(),
                cas_api_url: "https://api.cas.org".to_string(),
                api_keys: std::collections::HashMap::new(),
                cache_ttl_hours: 24,
                update_interval_hours: 168, // 1 week
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
                max_file_size: Some(100 * 1024 * 1024), // 100MB
                max_files: Some(10),
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                health_check_interval_seconds: 30,
                prometheus_namespace: "elementa".to_string(),
            },
        }
    }
}