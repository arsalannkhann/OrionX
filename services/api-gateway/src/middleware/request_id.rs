use axum::{
    http::{Request, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

const REQUEST_ID_HEADER: &str = "x-request-id";

pub async fn request_id_middleware(
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    // Generate or extract request ID
    let request_id = if let Some(existing_id) = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|v| v.to_str().ok())
    {
        existing_id.to_string()
    } else {
        let id = Uuid::new_v4().to_string();
        request.headers_mut().insert(
            REQUEST_ID_HEADER,
            HeaderValue::from_str(&id).unwrap(),
        );
        id
    };

    // Add request ID to tracing span
    let span = tracing::info_span!("request", request_id = %request_id);
    let _guard = span.enter();

    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        REQUEST_ID_HEADER,
        HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}