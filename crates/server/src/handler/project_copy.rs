use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_r_protocol::payload::{DataResponse, NoContent, ProjectCopyCreateInput};
use opencode_r_core::ProjectCopyCreateInput as CoreProjectCopyCreateInput;
use crate::SharedState;

pub async fn create(
    State(state): State<SharedState>,
    Path(project_id): Path<String>,
    Json(payload): Json<ProjectCopyCreateInput>,
) -> Result<Json<DataResponse<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    state.project_copy.create(CoreProjectCopyCreateInput {
        project_id,
        source_directory: payload.source_directory,
    })
    .map(|data| Json(DataResponse { data }))
    .map_err(|e| (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({"message": e, "forceRequired": false})),
    ))
}

pub async fn refresh(
    State(state): State<SharedState>,
    Path(project_id): Path<String>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.project_copy.refresh(&project_id)
        .map(|_| Json(NoContent))
        .map_err(|e| (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"message": e})),
        ))
}
