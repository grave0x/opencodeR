use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_protocol::payload::{DataResponse, NoContent, QuestionReplyInput};
use opencode_core::QuestionReplyInput as CoreQuestionReplyInput;
use opencode_schema::question::Question;
use crate::SharedState;

pub async fn request_list(State(state): State<SharedState>) -> Json<DataResponse<Vec<Question>>> {
    Json(DataResponse { data: state.question.request_list() })
}

pub async fn session_list(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
) -> Json<DataResponse<Vec<Question>>> {
    Json(DataResponse { data: state.question.session_list(&session_id) })
}

pub async fn session_reply(
    State(state): State<SharedState>,
    Path((session_id, request_id)): Path<(String, String)>,
    Json(payload): Json<QuestionReplyInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.question.session_reply(&session_id, &request_id, CoreQuestionReplyInput {
        answers: vec![payload.answer],
    }).map(|_| Json(NoContent))
    .map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"message": format!("Question request not found: {}", request_id)})),
    ))
}

pub async fn session_reject(
    State(state): State<SharedState>,
    Path((session_id, request_id)): Path<(String, String)>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.question.session_reject(&session_id, &request_id)
        .map(|_| Json(NoContent))
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("Question request not found: {}", request_id)})),
        ))
}
