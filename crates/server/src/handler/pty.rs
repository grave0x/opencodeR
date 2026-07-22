use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use opencode_r_protocol::payload::{
    DataResponse, NoContent,
    PtyCreateInput, PtyUpdateInput, PtyConnectTokenResponse,
};
use opencode_r_core::{PtyCreateInput as CorePtyCreateInput, PtyUpdateInput as CorePtyUpdateInput};
use opencode_r_schema::pty::PtyInfo;
use crate::SharedState;
use std::os::fd::{FromRawFd, IntoRawFd};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

pub async fn delete_pty(
    State(state): State<SharedState>,
    Path(pty_id): Path<String>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    if state.pty.delete(&pty_id) {
        Ok(Json(NoContent))
    } else {
        Err((StatusCode::NOT_FOUND, Json(serde_json::json!({"message": format!("PTY not found: {}", pty_id)}))))
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
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
    Path(pty_id): Path<String>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    // Validate connect_token
    let token = params.get("connect_token").ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({"error": "Missing connect_token"})),
    ))?;
    let valid = state.pty.verify_token(&pty_id, token);
    if !valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({"error": "Invalid connect_token"})),
        ));
    }
    Ok(ws.on_upgrade(move |socket| handle_pty_ws(socket, state, pty_id)))
}

async fn handle_pty_ws(mut ws: WebSocket, state: SharedState, pty_id: String) {
    // Send connected event
    let _ = ws.send(Message::Text(
        serde_json::json!({"type": "connected", "pty_id": &pty_id}).to_string()
    )).await;

    // Take ownership of PTY stdio
    let Some(stdio) = state.pty.attach_stdio(&pty_id) else {
        let _ = ws.send(Message::Text(
            serde_json::json!({"type": "error", "message": "PTY not found"}).to_string()
        )).await;
        return;
    };

    // Convert std pipes to tokio async handles via raw fd
    // SAFETY: ChildStdout/ChildStdin/ChildStderr are thin wrappers around fd's.
    // We own them exclusively and convert each exactly once.
    let out_fd = stdio.stdout.into_raw_fd();
    let in_fd = stdio.stdin.into_raw_fd();
    let err_fd = stdio.stderr.into_raw_fd();
    let mut async_out = unsafe { tokio::fs::File::from_raw_fd(out_fd) };
    let mut async_in = unsafe { tokio::fs::File::from_raw_fd(in_fd) };
    let mut async_err = unsafe { tokio::fs::File::from_raw_fd(err_fd) };
    let exit_rx = stdio.exit_rx;

    // Split WebSocket into sender (Sink) and receiver (Stream)
    let (ws_sender, mut ws_receiver) = ws.split();
    let ws_sender = Arc::new(tokio::sync::Mutex::new(ws_sender));

    // Read from PTY stdout → WebSocket sender (Binary messages)
    let snd = ws_sender.clone();
    let stdout_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            match async_out.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let mut sender = snd.lock().await;
                    if sender.send(Message::Binary(buf[..n].to_vec())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Read from PTY stderr → WebSocket sender (Text JSON)
    let snd = ws_sender.clone();
    let stderr_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 4096];
        loop {
            match async_err.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    let text = String::from_utf8_lossy(&buf[..n]);
                    let msg = serde_json::json!({"type": "stderr", "data": text});
                    let mut sender = snd.lock().await;
                    if sender.send(Message::Text(msg.to_string())).await.is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    // Read from WebSocket → PTY stdin
    let stdin_task = tokio::spawn(async move {
        while let Some(msg) = ws_receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => { let _ = async_in.write_all(text.as_bytes()).await; }
                Ok(Message::Binary(data)) => { let _ = async_in.write_all(&data).await; }
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
        let _ = async_in.shutdown().await;
    });

    // Wait for exit notification and send it
    let snd = ws_sender.clone();
    let exit_task = tokio::spawn(async move {
        if let Ok(code) = exit_rx.await {
            let msg = serde_json::json!({"type": "exit", "code": code});
            let mut sender = snd.lock().await;
            let _ = sender.send(Message::Text(msg.to_string())).await;
        }
    });

    // Wait for any task to finish, then cancel all others
    tokio::select! {
        _ = stdout_task => {},
        _ = stderr_task => {},
        _ = stdin_task => {},
        _ = exit_task => {},
    }

    // Clean up orphaned child process
    state.pty.delete(&pty_id);
}
