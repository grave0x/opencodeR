use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use opencode_protocol::payload::{
    CursorLinks, CursorResponse, DataResponse, NoContent,
    SessionActiveMap, SessionActive, SessionCreateInput,
    SessionSwitchAgentInput, SessionSwitchModelInput,
    SessionPromptInput, SessionRevertStageInput,
    RevertStageResponse, SessionContextResponse,
    SessionHistoryResponse,
};
use opencode_protocol::error::{
    SessionNotFoundError, MessageNotFoundError,
};
use opencode_protocol::query::{SessionsQuery, SessionHistoryQuery as QSessionHistoryQuery, SessionMessagesQuery};
use opencode_core::{
    SessionListQuery, SessionCreateInput as CoreCreateInput,
    SessionPromptInput as CorePromptInput,
    SessionRevertStageInput as CoreRevertStageInput,
    SessionHistoryQuery as CoreHistoryQuery,
    SessionMessagesQuery as CoreMessagesQuery,
};
use opencode_schema::session::{SessionInfo, ListDirection};
use opencode_schema::session_id::SessionID;
use opencode_schema::session_message::{SessionMessage, SessionMessageID};
use opencode_schema::session_event::SessionEvent;
use crate::SharedState;

/// Leetopt: encode cursor to base64url without serde_json Value allocation or format! overhead.
/// Known shape: {"id":"<26ch>","time":<int>,"direction":"previous|next"}
fn encode_cursor(id: &str, time: i64, direction: &str) -> String {
    let mut buf = Vec::with_capacity(96);
    buf.push(b'{');
    buf.extend_from_slice(b"\"id\":\"");
    buf.extend_from_slice(id.as_bytes());
    buf.extend_from_slice(b"\",\"time\":");
    // Leetopt: manual i64 formatting — no format!(), no itoa dependency
    write_i64(&mut buf, time);
    buf.extend_from_slice(b",\"direction\":\"");
    buf.extend_from_slice(direction.as_bytes());
    buf.push(b'"');
    buf.push(b'}');

    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&buf)
}

/// Leetopt: write an i64 decimal to a Vec<u8> without allocation or formatting machinery.
fn write_i64(buf: &mut Vec<u8>, n: i64) {
    if n == 0 {
        buf.push(b'0');
        return;
    }
    if n == i64::MIN {
        buf.extend_from_slice(b"-9223372036854775808");
        return;
    }
    let mut abs = if n < 0 {
        buf.push(b'-');
        (-n) as u64
    } else {
        n as u64
    };
    // Write digits in reverse into a temp buffer, then reverse
    let mut tmp = [0u8; 20];
    let mut i = 0;
    while abs > 0 {
        tmp[i] = b'0' + (abs % 10) as u8;
        abs /= 10;
        i += 1;
    }
    for j in (0..i).rev() {
        buf.push(tmp[j]);
    }
}

/// Leetopt: decode cursor without serde_json Value tree allocation.
/// Manual scanner on the decoded bytes — extracts the three fields by scanning
/// for known key patterns rather than building a full JSON DOM.
fn decode_cursor(cursor: &str) -> Option<(String, i64, String)> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(cursor).ok()?;
    let s = std::str::from_utf8(&bytes).ok()?;

    // Fast manual field extraction: find "id":" then read to next "
    let id = extract_string_field(s, "\"id\":\"")?;
    let time = extract_int_field(s, "\",\"time\":")?;
    let direction = extract_string_field(s, ",\"direction\":\"")?;

    Some((id, time, direction))
}

/// Extract a quoted string field: find `prefix`, then read until the next `"`.
fn extract_string_field<'a>(s: &'a str, prefix: &str) -> Option<String> {
    let start = s.find(prefix)? + prefix.len();
    let end = s[start..].find('"')?;
    Some(s[start..start + end].to_string())
}

/// Extract an integer field: find `prefix`, then read consecutive digit/minus chars.
fn extract_int_field(s: &str, prefix: &str) -> Option<i64> {
    let start = s.find(prefix)? + prefix.len();
    let end = s[start..].find(|c: char| !c.is_ascii_digit() && c != '-')?;
    s[start..start + end].parse::<i64>().ok()
}

pub async fn list(
    State(state): State<SharedState>,
    Query(query): Query<SessionsQuery>,
) -> Json<CursorResponse<Vec<SessionInfo>>> {
    let (cursor_id, cursor_time, cursor_direction) = query.cursor.as_ref()
        .and_then(|c| decode_cursor(&c.0))
        .map(|(id, time, dir)| (Some(id), Some(time), Some(dir)))
        .unwrap_or((None, None, None));

    let sessions = state.session.list(SessionListQuery {
        workspace: query.workspace,
        limit: query.limit.map(|l| l as u32),
        order: query.order.map(|o| match o {
            ListDirection::Previous => "desc".into(),
            ListDirection::Next => "asc".into(),
        }),
        search: query.search,
        directory: query.directory,
        project: query.project.map(|p| p.0),
        subpath: query.subpath,
        cursor: query.cursor.map(|c| c.0),
        cursor_id,
        cursor_time,
        cursor_direction,
    });
    let previous = sessions.first().map(|s|
        encode_cursor(&s.id.0, s.time.created.timestamp_millis(), "previous")
    );
    let next = sessions.last().map(|s|
        encode_cursor(&s.id.0, s.time.created.timestamp_millis(), "next")
    );
    Json(CursorResponse {
        data: sessions,
        cursor: CursorLinks { previous, next },
    })
}

pub async fn create(
    State(state): State<SharedState>,
    Json(payload): Json<SessionCreateInput>,
) -> Json<DataResponse<SessionInfo>> {
    let session = state.session.create(CoreCreateInput {
        id: payload.id,
        agent: payload.agent.map(|a| a.0),
        model: payload.model.map(|m| m.0),
        location: payload.location,
    });
    Json(DataResponse { data: session })
}

pub async fn active(State(state): State<SharedState>) -> Json<SessionActiveMap> {
    let sessions = state.session.active();
    Json(SessionActiveMap {
        sessions: sessions.into_iter().map(|(k, v)| {
            (k, SessionActive { r#type: v })
        }).collect(),
    })
}

pub async fn get(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<DataResponse<SessionInfo>>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    match state.session.get(&session_id) {
        Some(session) => Ok(Json(DataResponse { data: session })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        )),
    }
}

pub async fn switch_agent(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Json(payload): Json<SessionSwitchAgentInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.switch_agent(&session_id, &payload.agent.0)
        .map(|_| Json(NoContent))
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        ))
}

pub async fn switch_model(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Json(payload): Json<SessionSwitchModelInput>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.switch_model(&session_id, &payload.model.0)
        .map(|_| Json(NoContent))
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        ))
}

pub async fn prompt(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Json(payload): Json<SessionPromptInput>,
) -> Result<Json<DataResponse<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
    let sid = session_id.0.clone();
    match state.session.prompt(&session_id, CorePromptInput {
        id: payload.id,
        prompt: payload.prompt,
        delivery: payload.delivery,
        resume: payload.resume,
    }) {
        Ok(msg_id) => Ok(Json(DataResponse {
            data: serde_json::json!({"id": msg_id, "status": "admitted"}),
        })),
        Err(e) if e == "Session not found" => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }).unwrap()),
        )),
        Err(e) => Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"message": e})),
        )),
    }
}

pub async fn compact(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.session.compact(&session_id)
        .map(|_| Json(NoContent))
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": e})),
        ))
}

pub async fn wait(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.wait(&session_id)
        .map(|_| Json(NoContent))
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        ))
}

pub async fn revert_stage(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Json(payload): Json<SessionRevertStageInput>,
) -> Result<Json<RevertStageResponse>, (StatusCode, Json<serde_json::Value>)> {
    match state.session.revert_stage(&session_id, CoreRevertStageInput {
        message_id: payload.message_id,
        files: payload.files,
    }) {
        Ok(revert) => Ok(Json(RevertStageResponse { data: revert })),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": e})),
        )),
    }
}

pub async fn revert_clear(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<serde_json::Value>)> {
    state.session.revert_clear(&session_id)
        .map(|_| Json(NoContent))
        .map_err(|e| (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"message": e})),
        ))
}

pub async fn revert_commit(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.revert_commit(&session_id)
        .map(|_| Json(NoContent))
        .map_err(|_| (
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        ))
}

pub async fn context(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<SessionContextResponse>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    match state.session.context(&session_id) {
        Ok(messages) => Ok(Json(SessionContextResponse { data: messages })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        )),
    }
}

pub async fn history(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Query(query): Query<QSessionHistoryQuery>,
) -> Result<Json<SessionHistoryResponse>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    match state.session.history(&session_id, CoreHistoryQuery {
        limit: query.limit.map(|l| l as u32),
        after: query.after.map(|a| a as u32),
    }) {
        Ok(result) => Ok(Json(SessionHistoryResponse {
            data: result.events,
            has_more: result.has_more,
        })),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }),
        )),
    }
}

pub async fn events(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Query(query): Query<QSessionHistoryQuery>,
) -> Json<DataResponse<Vec<SessionEvent>>> {
    let events = state.session.events(&session_id, query.after.map(|a| a as u32));
    Json(DataResponse { data: events })
}

pub async fn interrupt(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Json<NoContent> {
    state.session.interrupt(&session_id);
    Json(NoContent)
}

pub async fn messages(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Query(query): Query<SessionMessagesQuery>,
) -> Result<Json<CursorResponse<Vec<SessionMessage>>>, (StatusCode, Json<SessionNotFoundError>)> {
    match state.session.messages(CoreMessagesQuery {
        session_id,
        limit: query.limit,
        order: query.order.map(|o| match o {
            ListDirection::Previous => "desc".into(),
            ListDirection::Next => "asc".into(),
        }),
        cursor: query.cursor,
    }) {
        Ok(msgs) => {
            let _first = msgs.first();
            let _last = msgs.last();
            Ok(Json(CursorResponse {
                data: msgs,
                cursor: CursorLinks {
                    previous: None,
                    next: None,
                },
            }))
        }
        Err(_) => Err((StatusCode::NOT_FOUND, Json(SessionNotFoundError {
            session_id: String::new(),
            message: "Session not found".into(),
        }))),
    }
}

pub async fn message(
    State(state): State<SharedState>,
    Path((session_id, message_id)): Path<(SessionID, SessionMessageID)>,
) -> Result<Json<DataResponse<SessionMessage>>, (StatusCode, Json<serde_json::Value>)> {
    let sid = session_id.0.clone();
    let mid = message_id.0.clone();
    match state.session.message(&session_id, &message_id) {
        Some(msg) => Ok(Json(DataResponse { data: msg })),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(MessageNotFoundError {
                session_id: sid,
                message_id: mid,
                message: format!("Message not found: {}", message_id.0),
            }).unwrap()),
        )),
    }
}
