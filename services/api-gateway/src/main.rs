use anyhow::Result;
use axum::{
    extract::DefaultBodyLimit,
    http::{header, Method},
    response::Json,
    routing::{get},
    serve, Router,
};
use elementa_database::initialize_databases;
use elementa_utils::{init_logging, AppConfig};
use serde_json::json;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
};
use tracing::info;

mod handlers;
mod middleware;
mod routes;

use middleware::*;


#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = AppConfig::load().unwrap_or_else(|_| {
        eprintln!("Failed to load configuration, using defaults");
        AppConfig::default()
    });

    // Initialize logging
    init_logging(&config.logging)?;
    info!("Starting Elementa API Gateway");

    // Initialize databases
    let db_config = elementa_database::DatabaseConfig {
        postgres_url: config.database.postgres_url.clone(),
        mongodb_url: config.database.mongodb_url.clone(),
        redis_url: config.database.redis_url.clone(),
        max_connections: config.database.max_connections,
        connection_timeout: std::time::Duration::from_secs(config.database.connection_timeout_seconds),
    };
    let (postgres_pool, mongo_client, redis_pool) = initialize_databases(&db_config).await?;
    info!("Database connections established");

    // Build application router
    let app = create_app(postgres_pool, mongo_client, redis_pool, &config).await?;

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let listener = TcpListener::bind(&addr).await?;
    info!("API Gateway listening on {}", addr);

    serve(listener, app).await?;

    Ok(())
}

async fn create_app(
    postgres_pool: elementa_database::PostgresPool,
    mongo_client: elementa_database::MongoClient,
    redis_pool: elementa_database::RedisPool,
    config: &AppConfig,
) -> Result<Router> {
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/metrics", get(metrics_handler))
        
        // API routes
        .nest("/api/v1", routes::create_api_routes())
        
        // Middleware stack
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
                        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
                )
                .layer(DefaultBodyLimit::max(config.server.max_request_size))
                .layer(axum::middleware::from_fn(request_id_middleware))
                .layer(axum::middleware::from_fn(error_handling_middleware))
        )
        
        // Application state
        .with_state(AppState {
            postgres_pool,
            mongo_client,
            redis_pool,
            config: config.clone(),
        });

    Ok(app)
}

#[derive(Clone)]
pub struct AppState {
    pub postgres_pool: elementa_database::PostgresPool,
    pub mongo_client: elementa_database::MongoClient,
    pub redis_pool: elementa_database::RedisPool,
    pub config: AppConfig,
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "elementa-api-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn metrics_handler() -> String {
    use prometheus::{TextEncoder};
    
    let encoder = TextEncoder::new();
    let metric_families = prometheus::gather();
    
    encoder.encode_to_string(&metric_families)
        .unwrap_or_else(|_| "Error encoding metrics".to_string())
}