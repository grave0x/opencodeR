use axum::{extract::State, Json};
use opencode_protocol::payload::DataResponse;
use opencode_schema::command::Command;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<Command>>> {
    Json(DataResponse { data: state.command.list() })
}
