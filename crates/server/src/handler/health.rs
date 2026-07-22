use axum::Json;
use opencode_protocol::payload::HealthResponse;

pub async fn get() -> Json<HealthResponse> {
    Json(HealthResponse { healthy: true })
}
