use axum::{extract::State, Json};
use opencode_protocol::payload::DataResponse;
use opencode_schema::skill::Skill;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<Skill>>> {
    Json(DataResponse { data: state.skill.list() })
}
