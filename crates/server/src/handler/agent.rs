use axum::{extract::State, Json};
use opencode_r_protocol::payload::DataResponse;
use opencode_r_schema::agent::AgentInfo;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<AgentInfo>>> {
    Json(DataResponse { data: state.agent.list() })
}
