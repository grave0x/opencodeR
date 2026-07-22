use axum::{extract::{Path, Query, State}, http::StatusCode, Json};
use axum::response::{
    sse::{Event, KeepAlive},
    Sse,
};
use opencode_r_protocol::payload::{
    CursorLinks, CursorResponse, DataResponse, NoContent,
    SessionActiveMap, SessionActive, SessionCreateInput,
    SessionSwitchAgentInput, SessionSwitchModelInput,
    SessionPromptInput, SessionRevertStageInput,
    RevertStageResponse, SessionContextResponse,
    SessionHistoryResponse,
};
use opencode_r_protocol::error::{
    SessionNotFoundError, MessageNotFoundError,
};
use opencode_r_protocol::query::{SessionsQuery, SessionHistoryQuery as QSessionHistoryQuery, SessionMessagesQuery};
use opencode_r_core::{
    SessionListQuery, SessionCreateInput as CoreCreateInput,
    SessionPromptInput as CorePromptInput,
    SessionRevertStageInput as CoreRevertStageInput,
    SessionHistoryQuery as CoreHistoryQuery,
    SessionMessagesQuery as CoreMessagesQuery,
};
use opencode_r_schema::session::{SessionInfo, ListDirection};
use opencode_r_schema::session_event::SessionEvent;
use opencode_r_schema::session_id::SessionID;
use opencode_r_schema::session_message::{SessionMessage, SessionMessageID};
use futures::stream::{self, Stream, StreamExt};
use std::convert::Infallible;
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

pub async fn delete_session(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.delete(&session_id).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
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
    let prompt_text = payload.prompt.clone();

    // 1. Admit the user prompt
    let msg_id = match state.session.prompt(&session_id, CorePromptInput {
        id: payload.id,
        prompt: payload.prompt,
        delivery: payload.delivery,
        resume: payload.resume,
    }) {
        Ok(id) => id,
        Err(e) if e == "Session not found" => return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(SessionNotFoundError {
                session_id: sid,
                message: format!("Session not found: {}", session_id.0),
            }).unwrap()),
        )),
        Err(e) => return Err((
            StatusCode::CONFLICT,
            Json(serde_json::json!({"message": e})),
        )),
    };

    // 2. Try to call LLM for a real response
    let llm_config = opencode_r_llm::ProviderConfig::from_env();
    if llm_config.is_configured() {
        // Get session to determine model
        let model = state.session.get(&session_id)
            .and_then(|s| s.model.map(|m| m.0))
            .unwrap_or_else(|| "anthropic/claude-sonnet-4".to_string());

        if let Some(provider) = opencode_r_llm::provider_for_model(&llm_config, &model) {
            match provider.complete(&prompt_text, &model.split('/').nth(1).unwrap_or(&model)) {
                Ok(response) => {
                    let preview = response.content[..100.min(response.content.len())].to_string();
                    let _ = state.session.push_message(&session_id,
                        opencode_r_schema::session_message::MessageRole::Assistant,
                        vec![opencode_r_schema::session_message::MessageContent::Text { text: response.content }],
                    );
                    return Ok(Json(DataResponse {
                        data: serde_json::json!({"id": msg_id, "status": "completed", "response_preview": preview}),
                    }));
                }
                Err(llm_err) => {
                    tracing::warn!(target: "opencode_r_server", session_id = %session_id.0, error = %llm_err, "LLM call failed");
                }
            }
        }
    }

    // 3. Fallback: return admitted without AI response
    Ok(Json(DataResponse {
        data: serde_json::json!({"id": msg_id, "status": "admitted"}),
    }))
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
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let after = query.after.map(|a| a as u32);

    // Replay past events
    let past = state.session.events(&session_id, after);
    let replay_stream = stream::iter(past.into_iter().map(|ev| {
        Ok(Event::default()
            .event("message")
            .json_data(ev)
            .unwrap())
    }));

    // Subscribe to live events
    let rx = state.session.subscribe_events();
    let sid = session_id.0.clone();
    let live_stream = tokio_stream::wrappers::BroadcastStream::new(rx)
        .filter_map(move |result| {
            let sid = sid.clone();
            async move {
                match result {
                    Ok(ev) if ev.session_id.0 == sid => {
                        Some(Ok(Event::default()
                            .event("message")
                            .json_data(ev)
                            .unwrap()))
                    }
                    _ => None, // wrong session or lagged — skip
                }
            }
        });

    let stream = replay_stream.chain(live_stream);
    Sse::new(stream).keep_alive(KeepAlive::new())
}

pub async fn interrupt(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Json<NoContent> {
    state.session.interrupt(&session_id);
    Json(NoContent)
}

pub async fn pause(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.pause(&session_id).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
}

pub async fn resume(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.resume(&session_id).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
}

pub async fn freeze(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.freeze(&session_id).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
}

pub async fn terminate(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    state.session.terminate(&session_id).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
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


pub async fn cost_summary(
    State(state): State<SharedState>,
) -> Json<DataResponse<opencode_r_schema::session::CostSummary>> {
    let summary = state.session.cost_summary();
    Json(DataResponse { data: summary })
}

pub async fn trace(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    let session = state.session.get(&session_id).ok_or_else(|| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid.clone(), message: format!("Session not found: {}", sid) }),
    ))?;
    let messages = state.session.messages(opencode_r_core::SessionMessagesQuery {
        session_id: session_id.clone(),
        limit: None,
        order: Some("asc".into()),
        cursor: None,
    }).unwrap_or_default();
    let events = state.session.events(&session_id, None);

    let trace = serde_json::json!({
        "session": session,
        "messages": messages,
        "events": events,
        "message_count": messages.len(),
        "event_count": events.len(),
    });
    Ok(Json(trace))
}

pub async fn audit_log(
    State(state): State<SharedState>,
    Query(query): Query<QSessionHistoryQuery>,
) -> Json<DataResponse<Vec<SessionEvent>>> {
    let events = state.session.global_events(query.after.map(|a| a as u32), query.limit.map(|l| l as u32));
    Json(DataResponse { data: events })
}

pub async fn set_group(
    State(state): State<SharedState>,
    Path(session_id): Path<SessionID>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<NoContent>, (StatusCode, Json<SessionNotFoundError>)> {
    let sid = session_id.0.clone();
    let group = payload.get("group").and_then(|g| g.as_str().map(String::from));
    state.session.set_group(&session_id, group).map(|_| Json(NoContent)).map_err(|_| (
        StatusCode::NOT_FOUND,
        Json(SessionNotFoundError { session_id: sid, message: format!("Session not found: {}", session_id.0) }),
    ))
}

pub async fn list_groups(
    State(state): State<SharedState>,
) -> Json<DataResponse<Vec<serde_json::Value>>> {
    let groups = state.session.list_groups();
    let data: Vec<serde_json::Value> = groups.into_iter().map(|(name, count)|
        serde_json::json!({"group": name, "count": count})
    ).collect();
    Json(DataResponse { data })
}
