pub mod conversation;
pub mod input;

use anyhow::Result;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Style};
use ratatui::widgets::Block;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use futures::StreamExt;
use std::time::Duration;

use opencode_r_schema::session_message::{MessageContent, MessageRole, SessionMessage};
use opencode_r_schema::session_id::SessionID;

pub struct AgentState {
    pub session_id: String,
    pub messages: Vec<SessionMessage>,
    pub scroll_offset: usize,
    pub input: String,
    pub status_msg: String,
}

impl AgentState {
    pub fn new(session_id: String) -> Self {
        Self { session_id, messages: Vec::new(), scroll_offset: 0, input: String::new(), status_msg: "Ready".into() }
    }
}

pub async fn run_agent(port: u16) -> Result<()> {
    let client = reqwest::Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);
    tokio::time::sleep(Duration::from_millis(500)).await;

    let resp = client.post(format!("{}/api/session", base_url))
        .json(&serde_json::json!({"agent": "build"}))
        .send().await?;
    let body: serde_json::Value = resp.json().await?;
    let sid = body["data"]["id"].as_str().unwrap_or("?").to_string();
    let mut state = AgentState::new(sid);
    refresh_messages(&client, &base_url, &mut state).await;

    let mut terminal = ratatui::init();
    crossterm::execute!(std::io::stderr(), crossterm::event::EnableMouseCapture)?;
    let mut reader = event::EventStream::new();
    let mut tick = tokio::time::interval(Duration::from_millis(100));

    loop {
        terminal.draw(|f| render(f, &state))?;

        tokio::select! {
            _ = tick.tick() => {}
            Some(Ok(evt)) = reader.next() => {
                match evt {
                    Event::Key(key) => {
                        if key.kind != KeyEventKind::Press { continue; }
                        match key.code {
                            KeyCode::Char(c) => state.input.push(c),
                            KeyCode::Backspace => { state.input.pop(); }
                            KeyCode::Enter => {
                                let input = state.input.trim().to_string();
                                if input.is_empty() { continue; }
                                if input == "/quit" || input == "/q" { break; }
                                state.input.clear();
                                state.status_msg = "Sending...".into();

                                let resp = client.post(format!("{}/api/session/{}/prompt", base_url, state.session_id))
                                    .json(&serde_json::json!({"prompt": input, "resume": false}))
                                    .send().await;
                                match resp {
                                    Ok(r) => {
                                        let body: serde_json::Value = r.json().await.unwrap_or_default();
                                        state.status_msg = if body["data"]["response_preview"].is_string() {
                                            "Waiting for response...".into()
                                        } else { "Admitted (no LLM key)".into() };
                                    }
                                    Err(e) => state.status_msg = format!("Error: {}", e),
                                }
                                refresh_messages(&client, &base_url, &mut state).await;
                                state.scroll_offset = 0;
                            }
                            KeyCode::Up => state.scroll_offset = state.scroll_offset.saturating_add(1),
                            KeyCode::Down => state.scroll_offset = state.scroll_offset.saturating_sub(1),
                            KeyCode::Esc => break,
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    crossterm::execute!(std::io::stderr(), crossterm::event::DisableMouseCapture)?;
    ratatui::restore();
    Ok(())
}

fn render(frame: &mut Frame, state: &AgentState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let sid_short = if state.session_id.len() > 12 { &state.session_id[..12] } else { &state.session_id };
    let status = format!(" opencodeR | {} | {} msgs | {}", sid_short, state.messages.len(), state.status_msg);
    let sb = Block::default().style(Style::default().bg(Color::Blue).fg(Color::White));
    frame.render_widget(ratatui::widgets::Paragraph::new(status).block(sb), chunks[0]);

    self::conversation::draw(frame, chunks[1], state);
    self::input::draw(frame, chunks[2], state);
}

pub async fn refresh_messages(client: &reqwest::Client, base_url: &str, state: &mut AgentState) {
    if let Ok(r) = client.get(format!("{}/api/session/{}/message", base_url, state.session_id)).send().await {
        if let Ok(body) = r.json::<serde_json::Value>().await {
            if let Some(msgs) = body["data"].as_array() {
                state.messages = msgs.iter().filter_map(|m| {
                    let id = m["id"].as_str()?;
                    let role_str = m["role"].as_str().unwrap_or("user");
                    let text = m["content"].as_array()
                        .and_then(|c| c.first())
                        .and_then(|c| c["text"].as_str())
                        .unwrap_or("");
                    Some(SessionMessage {
                        id: opencode_r_schema::session_message::SessionMessageID(id.to_string()),
                        session_id: SessionID(state.session_id.clone()),
                        role: match role_str {
                            "assistant" => MessageRole::Assistant,
                            "tool" => MessageRole::Tool,
                            _ => MessageRole::User,
                        },
                        content: vec![MessageContent::Text { text: text.to_string() }],
                        created_at: chrono::Utc::now(),
                    })
                }).collect();
            }
        }
    }
}
