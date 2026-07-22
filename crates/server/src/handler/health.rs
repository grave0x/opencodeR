use axum::Json;
use opencode_r_protocol::payload::HealthResponse;

pub async fn get() -> Json<HealthResponse> {
    Json(HealthResponse { healthy: true })
}
