use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_r_protocol::payload::{
    DataResponse, NoContent,
    IntegrationConnectKeyInput, IntegrationConnectOAuthInput, IntegrationAttemptCompleteInput,
};
use opencode_r_core::{
    ConnectKeyInput, ConnectOAuthInput,
};
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<opencode_r_schema::integration::Integration>>> {
    Json(DataResponse { data: state.integration.list() })
}

pub async fn get(
    State(state): State<SharedState>,
    Path(integration_id): Path<String>,
) -> Result<Json<DataResponse<opencode_r_schema::integration::Integration>>, (StatusCode, Json<serde_json::Value>)> {
    match state.integration.get(&integration_id) {
        Some(integration) => Ok(Json(DataResponse { data: integration })),
        None => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"message": "Integration not found"})))),
    }
}

pub async fn connect_key(
    State(state): State<SharedState>,
    Path(integration_id): Path<String>,
    Json(payload): Json<IntegrationConnectKeyInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.integration.connect_key(ConnectKeyInput {
        integration_id,
        key: payload.key,
        label: payload.label,
    }).map(|_| Json(NoContent))
    .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "Authentication failed"}))))
}

pub async fn connect_oauth(
    State(state): State<SharedState>,
    Path(integration_id): Path<String>,
    Json(payload): Json<IntegrationConnectOAuthInput>,
) -> Result<Json<DataResponse<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    state.integration.connect_oauth(ConnectOAuthInput {
        integration_id,
        method_id: payload.method_id,
        inputs: payload.inputs,
        label: payload.label,
    }).map(|data| Json(DataResponse { data }))
    .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": "Authentication failed"}))))
}

pub async fn attempt_status(
    State(state): State<SharedState>,
    Path(attempt_id): Path<String>,
) -> Json<DataResponse<serde_json::Value>> {
    Json(DataResponse {
        data: state.integration.attempt_status(&attempt_id)
            .unwrap_or(serde_json::json!({"status": "pending"})),
    })
}

pub async fn attempt_complete(
    State(state): State<SharedState>,
    Path(attempt_id): Path<String>,
    Json(payload): Json<IntegrationAttemptCompleteInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.integration.attempt_complete(&attempt_id, payload.code)
        .map(|_| Json(NoContent))
        .map_err(|msg| (StatusCode::BAD_REQUEST, Json(serde_json::json!({"message": msg}))))
}
