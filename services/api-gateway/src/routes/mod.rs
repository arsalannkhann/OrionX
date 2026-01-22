use axum::{routing::get, Router};

use crate::{handlers::*, AppState};

pub fn create_api_routes() -> Router<AppState> {
    Router::new()
        .route("/health/detailed", get(detailed_health_check))
        // TODO: Add other API routes as services are implemented
        // .nest("/suppliers", supplier_routes())
        // .nest("/components", component_routes())
        // .nest("/compliance", compliance_routes())
        // .nest("/workflows", workflow_routes())
        // .nest("/documents", document_routes())
}