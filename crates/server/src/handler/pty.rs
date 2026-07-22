use axum::{extract::{Path, State}, http::StatusCode, Json};
use opencode_r_protocol::payload::{
    DataResponse,
    PtyCreateInput, PtyUpdateInput, PtyConnectTokenResponse,
};
use opencode_r_core::{PtyCreateInput as CorePtyCreateInput, PtyUpdateInput as CorePtyUpdateInput};
use opencode_r_schema::pty::PtyInfo;
use crate::SharedState;

pub async fn list(State(state): State<SharedState>) -> Json<DataResponse<Vec<PtyInfo>>> {
    Json(DataResponse { data: state.pty.list() })
}

pub async fn create(
    State(state): State<SharedState>,
    Json(payload): Json<PtyCreateInput>,
) -> Json<DataResponse<PtyInfo>> {
    let pty = state.pty.create(CorePtyCreateInput {
        cols: payload.cols,
        rows: payload.rows,
        cwd: payload.cwd,
        command: payload.command,
    });
    Json(DataResponse { data: pty })
}

pub async fn get(
    State(state): State<SharedState>,
    Path(pty_id): Path<String>,
) -> Result<Json<DataResponse<PtyInfo>>, (StatusCode, Json<serde_json::Value>)> {
    match state.pty.get(&pty_id) {
        Some(pty) => Ok(Json(DataResponse { data: pty })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("PTY session not found: {}", pty_id)})),
        )),
    }
}

pub async fn update(
    State(state): State<SharedState>,
    Path(pty_id): Path<String>,
    Json(payload): Json<PtyUpdateInput>,
) -> Result<Json<DataResponse<PtyInfo>>, (StatusCode, Json<serde_json::Value>)> {
    match state.pty.update(&pty_id, CorePtyUpdateInput {
        cols: payload.cols,
        rows: payload.rows,
    }) {
        Some(pty) => Ok(Json(DataResponse { data: pty })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("PTY session not found: {}", pty_id)})),
        )),
    }
}

pub async fn connect_token(
    State(state): State<SharedState>,
    Path(pty_id): Path<String>,
) -> Result<Json<PtyConnectTokenResponse>, (StatusCode, Json<serde_json::Value>)> {
    match state.pty.connect_token(&pty_id) {
        Some(ticket) => Ok(Json(PtyConnectTokenResponse { data: ticket })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": format!("PTY session not found: {}", pty_id)})),
        )),
    }
}

pub async fn connect(
    ws: axum::extract::ws::WebSocketUpgrade,
    Path(pty_id): Path<String>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        use axum::extract::ws::Message;

        // Send connected event
        let connected = serde_json::json!({
            "type": "connected",
            "pty_id": pty_id,
        });
        let _ = socket.send(Message::Text(connected.to_string())).await;

        // Echo loop — receives messages and sends them back
        // In production, this would wire to the actual PTY process
        loop {
            match socket.recv().await {
                Some(Ok(Message::Text(text))) => {
                    let _ = socket.send(Message::Text(text)).await;
                }
                Some(Ok(Message::Binary(data))) => {
                    let _ = socket.send(Message::Binary(data)).await;
                }
                Some(Ok(Message::Close(_))) | None => break,
                _ => break,
            }
        }
    })
}
