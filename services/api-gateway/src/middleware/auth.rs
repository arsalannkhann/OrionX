use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use elementa_utils::ElementaError;

use crate::AppState;

pub async fn auth_middleware(
    State(_state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, ElementaError> {
    // Extract authorization header
    let auth_header = headers
        .get("authorization")
        .and_then(|header| header.to_str().ok());

    // For now, we'll implement a simple token-based auth
    // In production, this would validate JWT tokens or API keys
    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..]; // Remove "Bearer " prefix
            
            // TODO: Implement proper token validation
            if token == "development-token" || validate_token(token).await {
                Ok(next.run(request).await)
            } else {
                Err(ElementaError::Authentication {
                    message: "Invalid token".to_string(),
                })
            }
        }
        Some(_) => Err(ElementaError::Authentication {
            message: "Invalid authorization header format".to_string(),
        }),
        None => Err(ElementaError::Authentication {
            message: "Missing authorization header".to_string(),
        }),
    }
}

async fn validate_token(_token: &str) -> bool {
    // TODO: Implement proper token validation
    // This could involve:
    // - JWT token verification
    // - Database lookup for API keys
    // - Integration with external auth providers
    true
}