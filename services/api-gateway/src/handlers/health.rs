use axum::{extract::State, response::Json};
use elementa_database::{postgres_health_check, mongo_health_check, redis_health_check};
use serde_json::{json, Value};

use crate::AppState;

pub async fn detailed_health_check(State(state): State<AppState>) -> Json<Value> {
    let mut health_status = json!({
        "status": "healthy",
        "service": "elementa-api-gateway",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "version": env!("CARGO_PKG_VERSION"),
        "checks": {}
    });

    // Check PostgreSQL
    let postgres_status = match postgres_health_check(&state.postgres_pool).await {
        Ok(_) => json!({"status": "healthy", "message": "Connected"}),
        Err(e) => json!({"status": "unhealthy", "message": e.to_string()}),
    };
    health_status["checks"]["postgres"] = postgres_status;

    // Check MongoDB
    let mongo_status = match mongo_health_check(&state.mongo_client).await {
        Ok(_) => json!({"status": "healthy", "message": "Connected"}),
        Err(e) => json!({"status": "unhealthy", "message": e.to_string()}),
    };
    health_status["checks"]["mongodb"] = mongo_status;

    // Check Redis
    let mut redis_pool = state.redis_pool.clone();
    let redis_status = match redis_health_check(&mut redis_pool).await {
        Ok(_) => json!({"status": "healthy", "message": "Connected"}),
        Err(e) => json!({"status": "unhealthy", "message": e.to_string()}),
    };
    health_status["checks"]["redis"] = redis_status;

    // Determine overall status
    let all_healthy = health_status["checks"]
        .as_object()
        .unwrap()
        .values()
        .all(|check| check["status"] == "healthy");

    if !all_healthy {
        health_status["status"] = json!("degraded");
    }

    Json(health_status)
}