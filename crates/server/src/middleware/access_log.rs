use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use std::time::Instant;
use tracing::info;

/// Enhanced request logging middleware.
/// Logs: method, path, status, duration_ms for every request.
pub async fn access_log_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start = Instant::now();

    let response = next.run(request).await;

    let status = response.status();
    let duration_ms = start.elapsed().as_millis();

    if status == StatusCode::NOT_FOUND {
        info!(
            target: "opencode_r_server::access",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration_ms,
            "NOT_FOUND"
        );
    } else if status.is_server_error() {
        info!(
            target: "opencode_r_server::access",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration_ms,
            "SERVER_ERROR"
        );
    } else {
        info!(
            target: "opencode_r_server::access",
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration_ms,
        );
    }

    response
}
