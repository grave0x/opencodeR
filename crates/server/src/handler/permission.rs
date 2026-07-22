use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_protocol::payload::{
    DataResponse, NoContent,
    SessionCreatePermissionInput, SessionPermissionReplyInput,
    PermissionCreateResponse,
};
use opencode_core::{
    PermissionCreateInput, PermissionReplyInput,
};
use crate::SharedState;

pub async fn request_list(State(state): State<SharedState>) -> Json<DataResponse<Vec<serde_json::Value>>> {
    Json(DataResponse { data: state.permission.request_list() })
}

pub async fn saved_list(
    State(state): State<SharedState>,
) -> Json<DataResponse<Vec<serde_json::Value>>> {
    Json(DataResponse { data: state.permission.saved_list(None) })
}

pub async fn session_list(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
) -> Json<DataResponse<Vec<serde_json::Value>>> {
    Json(DataResponse { data: state.permission.session_list(&session_id) })
}

pub async fn session_create(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    Json(payload): Json<SessionCreatePermissionInput>,
) -> Json<PermissionCreateResponse> {
    let result = state.permission.session_create(PermissionCreateInput {
        id: payload.id,
        session_id,
        action: payload.action,
        resources: payload.resources,
        save: payload.save,
        metadata: payload.metadata,
        source: payload.source,
        agent: payload.agent.map(|a| a.0),
    });
    Json(serde_json::from_value(result).unwrap_or(PermissionCreateResponse {
        id: "unknown".into(),
        effect: "deny".into(),
    }))
}

pub async fn session_get(
    State(state): State<SharedState>,
    Path((session_id, request_id)): Path<(String, String)>,
) -> Result<Json<DataResponse<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    match state.permission.session_get(&session_id, &request_id) {
        Some(data) => Ok(Json(DataResponse { data })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("Permission request not found: {}", request_id)})),
        )),
    }
}

pub async fn session_reply(
    State(state): State<SharedState>,
    Path((session_id, request_id)): Path<(String, String)>,
    Json(payload): Json<SessionPermissionReplyInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.permission.session_reply(&session_id, &request_id, PermissionReplyInput {
        reply: payload.reply,
        message: payload.message,
    }).map(|_| Json(NoContent))
    .map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"message": format!("Permission request not found: {}", request_id)})),
    ))
}
