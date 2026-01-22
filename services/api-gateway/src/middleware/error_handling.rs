use axum::{
    http::Request,
    middleware::Next,
    response::Response,
};

pub async fn error_handling_middleware(
    request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    next.run(request).await
}