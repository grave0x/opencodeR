use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_protocol::payload::{CredentialUpdateInput, NoContent};
use opencode_core::CredentialUpdateInput as CoreCredentialUpdateInput;
use crate::SharedState;

pub async fn update(
    State(state): State<SharedState>,
    Path(credential_id): Path<String>,
    Json(payload): Json<CredentialUpdateInput>,
) -> Result<Json<NoContent>, StatusCode> {
    state.credential.update(&credential_id, CoreCredentialUpdateInput { label: payload.label })
        .map(|_| Json(NoContent))
        .map_err(|_| StatusCode::NOT_FOUND)
}
