use axum::{extract::State, Json};
use opencode_r_protocol::payload::DataResponse;
use opencode_r_schema::reference::Reference;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<Reference>>> {
    Json(DataResponse { data: state.reference.list() })
}
