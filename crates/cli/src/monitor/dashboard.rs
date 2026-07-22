use std::collections::HashSet;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
    MouseEventKind,
};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, TableState},
    DefaultTerminal, Frame,
};
use tokio::sync::mpsc;

use super::client::DaemonStatus;
use super::client::MonitorClient;
use opencode_r_schema::session::{SessionInfo, SessionStatus};
use opencode_r_schema::session_event::SessionEvent;
use opencode_r_schema::session_message::SessionMessage;

// ── App State ────────────────────────────────────

enum EventLoopMsg {
    ServerEvent(SessionEvent),
    RefreshSessions,
}

enum Screen {
    Dashboard,
    Detail {
        session_id: String,
        messages: Vec<SessionMessage>,
        scroll: usize,
    },
}

struct AppState {
    screen: Screen,
    sessions: Vec<SessionInfo>,
    table_state: TableState,
    table_area: Rect,
    status: Option<DaemonStatus>,
    events: Vec<String>,
    filter_query: String,
    filter_active: bool,
    selected_set: HashSet<usize>,
    last_click_row: Option<usize>,
}

impl AppState {
    fn new() -> Self {
        Self {
            screen: Screen::Dashboard,
            sessions: vec![],
            table_state: TableState::default().with_offset(0),
            table_area: Rect::default(),
            status: None,
            events: Vec::with_capacity(128),
            filter_query: String::new(),
            filter_active: false,
            selected_set: HashSet::new(),
            last_click_row: None,
        }
    }

    fn selected_id(&self) -> Option<&str> {
        self.table_state
            .selected()
            .and_then(|i| self.sessions.get(i))
            .map(|s| s.id.0.as_str())
    }

    fn selected_or_current_ids(&self) -> Vec<String> {
        if self.selected_set.is_empty() {
            self.table_state
                .selected()
                .and_then(|i| self.sessions.get(i))
                .map(|s| vec![s.id.0.clone()])
                .unwrap_or_default()
        } else {
            self.selected_set
                .iter()
                .filter_map(|&i| self.sessions.get(i))
                .map(|s| s.id.0.clone())
                .collect()
        }
    }

    #[allow(dead_code)]
    fn selected_index(&self) -> Option<usize> {
        self.table_state.selected()
    }

    fn next_session(&mut self) {
        let i = self.table_state.selected().map_or(0, |i| {
            if i >= self.sessions.len().saturating_sub(1) {
                0
            } else {
                i + 1
            }
        });
        self.table_state.select(Some(i));
    }

    fn prev_session(&mut self) {
        let i = self.table_state.selected().map_or(0, |i| {
            if i == 0 {
                self.sessions.len().saturating_sub(1)
            } else {
                i - 1
            }
        });
        self.table_state.select(Some(i));
    }
}

// ── TUI Entry ────────────────────────────────────

pub async fn run(port: u16) -> Result<()> {
    let client = MonitorClient::new(port);
    let mut terminal = ratatui::init();
    crossterm::execute!(std::io::stderr(), EnableMouseCapture)?;
    let res = run_app(&mut terminal, &client).await;
    crossterm::execute!(std::io::stderr(), DisableMouseCapture)?;
    ratatui::restore();
    res
}

async fn run_app(terminal: &mut DefaultTerminal, client: &MonitorClient) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<EventLoopMsg>(256);
    let mut state = AppState::new();

    let mut event_stream = client.event_stream().await?;

    if let Ok(sessions) = client.list_sessions().await {
        state.sessions = sessions;
    }
    if let Ok(status) = client.status().await {
        state.status = Some(status);
    }

    let mut tick = tokio::time::interval(Duration::from_secs(2));
    let mut crossterm_reader = event::EventStream::new();

    let tx_for_key = tx.clone();
    let client_for_key = client.clone();

    loop {
        terminal.draw(|f| render(f, &mut state))?;

        tokio::select! {
            _ = tick.tick() => {
                if matches!(state.screen, Screen::Dashboard) {
                    let q = if state.filter_active || !state.filter_query.is_empty() {
                        Some(state.filter_query.as_str())
                    } else {
                        None
                    };
                    if let Ok(sessions) = client.list_sessions_filtered(q).await {
                        state.sessions = sessions;
                    }
                    if let Ok(status) = client.status().await {
                        state.status = Some(status);
                    }
                }

                if let Ok(Some(Ok(evt))) =
                    tokio::time::timeout(Duration::from_millis(500), event_stream.next()).await
                {
                    let _ = tx.send(EventLoopMsg::ServerEvent(evt)).await;
                }
            }
            Some(Ok(evt)) = crossterm_reader.next() => {
                match evt {
                    Event::Key(key) => {
                        if key.kind == KeyEventKind::Press
                            && handle_key(&key.code, &mut state, &client_for_key, &tx_for_key).await {
                                break; // quit
                            }
                    }
                    Event::Mouse(mouse) => {
                        handle_mouse(&mouse, &mut state, &client_for_key).await;
                    }
                    _ => {}
                }
            }
            Some(msg) = rx.recv() => {
                match msg {
                    EventLoopMsg::ServerEvent(evt) => {
                        let summary = format!(
                            "[{}] session {}",
                            event_kind_label(&evt.kind),
                            evt.session_id.0,
                        );
                        state.events.push(summary);
                        if state.events.len() > 128 {
                            state.events.remove(0);
                        }
                    }
                    EventLoopMsg::RefreshSessions => {
                        let q = if state.filter_active || !state.filter_query.is_empty() {
                            Some(state.filter_query.as_str())
                        } else {
                            None
                        };
                        if let Ok(sessions) = client.list_sessions_filtered(q).await {
                            state.sessions = sessions;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Key Handler ──────────────────────────────────

async fn handle_key(
    code: &KeyCode,
    state: &mut AppState,
    client: &MonitorClient,
    tx: &tokio::sync::mpsc::Sender<EventLoopMsg>,
) -> bool {
    // Search mode — capture input before normal key handling
    if state.filter_active {
        match code {
            KeyCode::Esc => {
                state.filter_active = false;
                state.filter_query.clear();
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Enter => {
                state.filter_active = false;
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Backspace => {
                state.filter_query.pop();
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Char(c) => {
                state.filter_query.push(*c);
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            _ => {}
        }
        return false;
    }

    match &state.screen {
        Screen::Dashboard => match code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Down | KeyCode::Char('j') => state.next_session(),
            KeyCode::Up | KeyCode::Char('k') => state.prev_session(),
            KeyCode::Char('/') => {
                state.filter_active = true;
            }
            KeyCode::Char(' ') => {
                if let Some(i) = state.table_state.selected() {
                    if state.selected_set.contains(&i) {
                        state.selected_set.remove(&i);
                    } else {
                        state.selected_set.insert(i);
                    }
                }
            }
            KeyCode::Enter => {
                if let Some(id) = state.selected_id().map(|s| s.to_string()) {
                    if let Ok(messages) = client.get_session_messages(&id).await {
                        state.screen = Screen::Detail {
                            session_id: id,
                            messages,
                            scroll: 0,
                        };
                    }
                }
            }
            KeyCode::Char('p') => {
                for id in state.selected_or_current_ids() {
                    let _ = client.pause(&id).await;
                }
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Char('r') => {
                for id in state.selected_or_current_ids() {
                    let _ = client.resume(&id).await;
                }
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Char('f') => {
                for id in state.selected_or_current_ids() {
                    let _ = client.freeze(&id).await;
                }
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            KeyCode::Char('t') => {
                for id in state.selected_or_current_ids() {
                    let _ = client.terminate(&id).await;
                }
                let _ = tx.send(EventLoopMsg::RefreshSessions).await;
            }
            _ => {}
        },
        Screen::Detail {
            session_id,
            messages,
            ..
        } => match code {
            KeyCode::Esc | KeyCode::Char('q') => {
                state.screen = Screen::Dashboard;
            }
            KeyCode::Char('e') => {
                let sid = session_id.clone();
                let msgs = messages.clone();
                let session = state.sessions.iter().find(|s| s.id.0 == sid).cloned();
                export_session(session.as_ref(), &msgs, "json");
                state.events.push(format!(
                    "[export] {} → {}.json ({} msgs)",
                    &sid[..8.min(sid.len())],
                    &sid[..8.min(sid.len())],
                    msgs.len()
                ));
            }
            KeyCode::Char('E') => {
                let sid = session_id.clone();
                let msgs = messages.clone();
                let session = state.sessions.iter().find(|s| s.id.0 == sid).cloned();
                for fmt in &["json", "yaml", "xml", "md"] {
                    export_session(session.as_ref(), &msgs, fmt);
                }
                state.events.push(format!(
                    "[export] {} → all formats ({} msgs)",
                    &sid[..8.min(sid.len())],
                    msgs.len()
                ));
            }
            _ => {}
        },
    }
    false
}

// ── Mouse Handler ────────────────────────────────

async fn handle_mouse(
    mouse: &crossterm::event::MouseEvent,
    state: &mut AppState,
    client: &MonitorClient,
) {
    match &state.screen {
        Screen::Dashboard => {
            let row_idx = hit_test_table_row(mouse, &state.table_area, state.sessions.len());

            match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if let Some(idx) = row_idx {
                        if mouse
                            .modifiers
                            .contains(crossterm::event::KeyModifiers::SHIFT)
                        {
                            if let Some(last) = state.last_click_row {
                                let range = if last <= idx { last..=idx } else { idx..=last };
                                state.selected_set.extend(range);
                            } else {
                                state.selected_set.insert(idx);
                            }
                        } else {
                            state.selected_set.clear();
                            state.selected_set.insert(idx);
                        }
                        state.table_state.select(Some(idx));
                        state.last_click_row = Some(idx);
                    }
                }
                MouseEventKind::Down(MouseButton::Right)
                    if row_idx.is_some() && state.selected_id().is_some() =>
                {
                    if let Some(id) = state.selected_id().map(|s| s.to_string()) {
                        if let Ok(messages) = client.get_session_messages(&id).await {
                            state.screen = Screen::Detail {
                                session_id: id,
                                messages,
                                scroll: 0,
                            };
                        }
                    }
                }
                _ => {}
            }
        }
        Screen::Detail { .. } => if mouse.kind == MouseEventKind::Down(MouseButton::Left) {},
    }
}

fn hit_test_table_row(
    mouse: &crossterm::event::MouseEvent,
    area: &Rect,
    row_count: usize,
) -> Option<usize> {
    if !hit_test_table(mouse, area) {
        return None;
    }
    let header_height = 2;
    let row_h = 1;
    let rel_y = mouse.row.saturating_sub(area.y);
    if rel_y < header_height {
        return None;
    }
    let row_idx = (rel_y - header_height) as usize / row_h;
    if row_idx < row_count {
        Some(row_idx)
    } else {
        None
    }
}

fn hit_test_table(mouse: &crossterm::event::MouseEvent, area: &Rect) -> bool {
    let (mx, my) = (mouse.column, mouse.row);
    mx >= area.x && mx < (area.x + area.width) && my >= area.y && my < (area.y + area.height)
}

// ── Rendering ────────────────────────────────────

fn render(frame: &mut Frame, state: &mut AppState) {
    match &state.screen {
        Screen::Dashboard => render_dashboard(frame, state),
        Screen::Detail {
            session_id,
            messages,
            scroll,
        } => render_detail(frame, state, session_id, messages, *scroll),
    }
}

// ── Dashboard rendering ──────────────────────────

fn render_dashboard(frame: &mut Frame, state: &mut AppState) {
    let has_filter = state.filter_active || !state.filter_query.is_empty();
    let constraints: Vec<Constraint> = if has_filter {
        vec![
            Constraint::Length(3), // status bar
            Constraint::Length(3), // search bar
            Constraint::Min(0),    // table
            Constraint::Length(9), // help
        ]
    } else {
        vec![
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(9),
        ]
    };
    let areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    if has_filter {
        let [top, search_area, mid, bottom] = [areas[0], areas[1], areas[2], areas[3]];
        state.table_area = mid;
        render_status_bar(frame, top, state);
        render_search_bar(frame, search_area, state);
        render_sessions_table(frame, mid, state);
        render_help_bar(frame, bottom, state);
    } else {
        let [top, mid, bottom] = [areas[0], areas[1], areas[2]];
        state.table_area = mid;
        render_status_bar(frame, top, state);
        render_sessions_table(frame, mid, state);
        render_help_bar(frame, bottom, state);
    }
}

fn render_status_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    // Compute aggregate stats
    let total_in: f64 = state.sessions.iter().map(|s| s.tokens.input).sum();
    let total_out: f64 = state.sessions.iter().map(|s| s.tokens.output).sum();
    let estimated_cost = estimate_cost(&state.sessions);
    let active = count_by_status(&state.sessions, SessionStatus::Active);
    let paused = count_by_status(&state.sessions, SessionStatus::Paused);
    let frozen = count_by_status(&state.sessions, SessionStatus::Frozen);

    let health = match &state.status {
        Some(s) if s.healthy => "●".to_string(),
        Some(_) => "○".to_string(),
        None => "◌".to_string(),
    };

    let stats = format!(
        " {} {} sessions | ${:.2} est | {}in/{}out | {}a {}p {}f ",
        health,
        state.sessions.len(),
        estimated_cost,
        format_tokens(total_in),
        format_tokens(total_out),
        active,
        paused,
        frozen,
    );

    let title = format!(" opencodeR —{} ", stats);

    let block = Block::default()
        .title(title)
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, area);
}

fn render_search_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let cursor = if state.filter_active { "█" } else { "" };
    let query = format!("/{}", state.filter_query);
    let label = if state.filter_active {
        format!(" Search: {query}{cursor} ")
    } else {
        format!(" Filter: {query} (Esc to clear) ")
    };
    let style = if state.filter_active {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    let block = Block::default()
        .title(label)
        .borders(Borders::ALL)
        .border_style(style);
    frame.render_widget(block, area);
}

fn render_sessions_table(frame: &mut Frame, area: Rect, state: &mut AppState) {
    let has_selection = !state.selected_set.is_empty();

    let header = Row::new(vec![
        "ID", "Title", "Model", "Status", "In Tok", "Out Tok", "Cost",
    ])
    .style(Style::default().add_modifier(Modifier::BOLD))
    .height(1);

    let rows: Vec<Row> = state
        .sessions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let prefix = if state.selected_set.contains(&i) {
                "●"
            } else {
                " "
            };
            let id_short = if s.id.0.len() > 7 {
                format!("{}{}", prefix, &s.id.0[..7])
            } else {
                format!("{}{}", prefix, s.id.0)
            };
            let title_short = if s.title.len() > 28 {
                format!("{}…", &s.title[..28])
            } else {
                s.title.clone()
            };
            let model_name = s
                .model
                .as_ref()
                .map(|m| {
                    if m.0.len() > 16 {
                        format!("{}…", &m.0[..16])
                    } else {
                        m.0.clone()
                    }
                })
                .unwrap_or_else(|| "—".to_string());
            let status = status_label(&s.status);
            let status_st = status_style(&s.status);
            let cost = format!("${:.4}", s.cost);
            let in_tok = format_tokens(s.tokens.input);
            let out_tok = format_tokens(s.tokens.output);
            Row::new(vec![
                Cell::from(Span::raw(id_short)),
                Cell::from(Span::raw(title_short)),
                Cell::from(Span::raw(model_name)),
                Cell::from(Span::styled(status, status_st)),
                Cell::from(Span::raw(in_tok)),
                Cell::from(Span::raw(out_tok)),
                Cell::from(Span::raw(cost)),
            ])
            .height(1)
        })
        .collect();

    if rows.is_empty() {
        let p = Paragraph::new("No sessions found.\nPress Enter on a session to view details.")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(p, area);
        return;
    }

    let widths = [
        Constraint::Length(10),
        Constraint::Min(18),
        Constraint::Length(18),
        Constraint::Length(12),
        Constraint::Length(8),
        Constraint::Length(8),
        Constraint::Length(10),
    ];

    let title = if has_selection {
        format!(" Sessions ({} selected) ", state.selected_set.len())
    } else {
        " Sessions ".to_string()
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(Style::default().fg(Color::Green)),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(table, area, &mut state.table_state);
}

fn render_help_bar(frame: &mut Frame, area: Rect, state: &AppState) {
    let [events_area, hints_area] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(2)])
        .split(area)
    else {
        return;
    };

    let items: Vec<ListItem> = state
        .events
        .iter()
        .rev()
        .take(events_area.height as usize - 2)
        .map(|e| ListItem::new(Line::from(Span::raw(e))))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Events ")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(list, events_area);

    let hints = vec![
        Line::from(vec![
            Span::styled(" j/k ", Style::default().fg(Color::DarkGray)),
            Span::raw("nav  "),
            Span::styled(" Space ", Style::default().fg(Color::Magenta)),
            Span::raw("select  "),
            Span::styled(" / ", Style::default().fg(Color::Yellow)),
            Span::raw("filter  "),
            Span::styled(" Enter ", Style::default().fg(Color::Cyan)),
            Span::raw("detail  "),
            Span::styled(" p ", Style::default().fg(Color::Yellow)),
            Span::raw("pause  "),
            Span::styled(" r ", Style::default().fg(Color::Green)),
            Span::raw("resume  "),
            Span::styled(" f ", Style::default().fg(Color::Blue)),
            Span::raw("freeze  "),
            Span::styled(" t ", Style::default().fg(Color::Red)),
            Span::raw("term  "),
            Span::styled(" q ", Style::default().fg(Color::DarkGray)),
            Span::raw("quit"),
        ]),
        Line::from(vec![
            Span::styled(" click ", Style::default().fg(Color::DarkGray)),
            Span::raw("select  "),
            Span::styled(" right-click ", Style::default().fg(Color::DarkGray)),
            Span::raw("detail"),
        ]),
    ];
    let p = Paragraph::new(hints).centered();
    frame.render_widget(p, hints_area);
}

// ── Detail page rendering ────────────────────────

fn render_detail(
    frame: &mut Frame,
    state: &AppState,
    session_id: &str,
    messages: &[SessionMessage],
    _scroll: usize,
) {
    let [top, mid, bottom] = *Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // breadcrumb
            Constraint::Min(0),    // messages
            Constraint::Length(1), // hint
        ])
        .split(frame.area())
    else {
        return;
    };

    // Breadcrumb
    let session = state.sessions.iter().find(|s| s.id.0 == session_id);
    let title = if let Some(s) = &session {
        format!(
            " {} — {} — {} — ${:.4} — {} ",
            &s.id.0[..8.min(s.id.0.len())],
            s.title,
            status_label(&s.status),
            s.cost,
            s.model.as_ref().map(|m| m.0.as_str()).unwrap_or("no model"),
        )
    } else {
        format!(" {} ", session_id)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    frame.render_widget(block, top);

    // Messages
    let msg_items: Vec<ListItem> = messages
        .iter()
        .map(|m| {
            let role_style = match m.role {
                opencode_r_schema::session_message::MessageRole::User => {
                    Style::default().fg(Color::Green)
                }
                opencode_r_schema::session_message::MessageRole::Assistant => {
                    Style::default().fg(Color::Cyan)
                }
                opencode_r_schema::session_message::MessageRole::Tool => {
                    Style::default().fg(Color::Yellow)
                }
                opencode_r_schema::session_message::MessageRole::System => {
                    Style::default().fg(Color::Magenta)
                }
            };
            let role_label = match m.role {
                opencode_r_schema::session_message::MessageRole::User => " YOU ",
                opencode_r_schema::session_message::MessageRole::Assistant => " AI  ",
                opencode_r_schema::session_message::MessageRole::Tool => " TOOL",
                opencode_r_schema::session_message::MessageRole::System => " SYS ",
            };

            let content_preview: String = m
                .content
                .iter()
                .map(|c| match c {
                    opencode_r_schema::session_message::MessageContent::Text { text } => {
                        text.chars().take(120).collect()
                    }
                    opencode_r_schema::session_message::MessageContent::ToolCall {
                        name, ..
                    } => {
                        format!("[tool:{}]", name)
                    }
                    opencode_r_schema::session_message::MessageContent::ToolResult {
                        content, ..
                    } => {
                        format!("[result: {}]", &content[..80.min(content.len())])
                    }
                })
                .collect::<Vec<_>>()
                .join(" | ");

            let line = Line::from(vec![
                Span::styled(role_label, role_style.add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(content_preview),
            ]);
            ListItem::new(line)
        })
        .collect();

    if msg_items.is_empty() {
        let p = Paragraph::new("No messages in this session.")
            .style(Style::default().fg(Color::DarkGray))
            .centered();
        frame.render_widget(p, mid);
    } else {
        let msg_list = List::new(msg_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Messages ({}) ", messages.len())),
            )
            .style(Style::default());
        frame.render_widget(msg_list, mid);
    }

    // Hint
    let hint = Line::from(vec![
        Span::styled(" Esc/q ", Style::default().fg(Color::DarkGray)),
        Span::raw("back  "),
        Span::styled(" e ", Style::default().fg(Color::Cyan)),
        Span::raw("export JSON  "),
        Span::styled(" E ", Style::default().fg(Color::Cyan)),
        Span::raw("export all"),
    ]);
    frame.render_widget(Paragraph::new(hint).centered(), bottom);
}

// ── Helpers ──────────────────────────────────────

fn status_label(status: &SessionStatus) -> String {
    match status {
        SessionStatus::Active => "active".into(),
        SessionStatus::Paused => "paused".into(),
        SessionStatus::Frozen => "frozen".into(),
        SessionStatus::Terminated => "terminated".into(),
    }
}

fn status_style(status: &SessionStatus) -> Style {
    match status {
        SessionStatus::Active => Style::default().fg(Color::Green),
        SessionStatus::Paused => Style::default().fg(Color::Yellow),
        SessionStatus::Frozen => Style::default().fg(Color::Blue),
        SessionStatus::Terminated => Style::default().fg(Color::Red),
    }
}

fn format_tokens(n: f64) -> String {
    if n >= 1_000_000.0 {
        format!("{:.1}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("{:.1}k", n / 1_000.0)
    } else {
        format!("{:.0}", n)
    }
}

fn event_kind_label(kind: &opencode_r_schema::session_event::SessionEventKind) -> &'static str {
    use opencode_r_schema::session_event::SessionEventKind;
    match kind {
        SessionEventKind::MessageAdded => "msg_added",
        SessionEventKind::MessageUpdated => "msg_updated",
        SessionEventKind::ToolCall => "tool_call",
        SessionEventKind::ToolResult => "tool_result",
        SessionEventKind::SessionCreated => "created",
        SessionEventKind::SessionArchived => "archived",
    }
}

fn count_by_status(sessions: &[SessionInfo], status: SessionStatus) -> usize {
    sessions.iter().filter(|s| s.status == status).count()
}

/// Estimate cost based on token usage and known provider pricing (per 1M tokens).
/// Falls back to session.cost if available, otherwise estimates.
fn estimate_cost(sessions: &[SessionInfo]) -> f64 {
    sessions
        .iter()
        .map(|s| {
            if s.cost > 0.0 {
                return s.cost;
            }
            // Default pricing estimates (USD per 1M tokens)
            let (input_rate, output_rate): (f64, f64) = match s.model.as_ref().map(|m| m.0.as_str())
            {
                Some(m) if m.contains("claude") => (3.0, 15.0),
                Some(m) if m.contains("gpt-5") => (1.25, 10.0),
                Some(m) if m.contains("gpt-4") => (2.5, 10.0),
                Some(m) if m.contains("gemini") => (0.5, 1.5),
                Some(m) if m.contains("deepseek") => (0.27, 1.10),
                Some(m) if m.contains("kimi") => (0.4, 1.6),
                _ => (1.0, 5.0), // default conservative estimate
            };
            (s.tokens.input / 1_000_000.0) * input_rate
                + (s.tokens.output / 1_000_000.0) * output_rate
        })
        .sum()
}

fn export_session(session: Option<&SessionInfo>, messages: &[SessionMessage], format: &str) {
    let dir = dirs_next().unwrap_or_else(|| std::path::PathBuf::from("."));
    std::fs::create_dir_all(&dir).ok();
    let dir_str = dir.display().to_string();

    let id_short = session
        .map(|s| &s.id.0[..8.min(s.id.0.len())])
        .unwrap_or("session");
    let ts = chrono::Utc::now().format("%Y%m%d-%H%M%S");
    let filename = format!("{dir_str}/{id_short}-{ts}.{format}");

    let content = match format {
        "json" => {
            #[derive(serde::Serialize)]
            struct Export<'a> {
                session: Option<&'a SessionInfo>,
                messages: &'a [SessionMessage],
            }
            serde_json::to_string_pretty(&Export { session, messages }).unwrap_or_default()
        }
        "yaml" => {
            #[derive(serde::Serialize)]
            struct Export<'a> {
                session: Option<&'a SessionInfo>,
                messages: &'a [SessionMessage],
            }
            serde_yaml::to_string(&Export { session, messages }).unwrap_or_default()
        }
        "xml" => {
            let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<session>\n");
            if let Some(s) = session {
                xml.push_str(&format!(
                    "  <id>{}</id>\n  <title>{}</title>\n  <status>{}</status>\n  <cost>{}</cost>\n",
                    s.id.0, s.title, status_label(&s.status), s.cost
                ));
            }
            xml.push_str("  <messages>\n");
            for m in messages {
                let role = format!("{:?}", m.role).to_lowercase();
                let text: String = m
                    .content
                    .iter()
                    .map(|c| match c {
                        opencode_r_schema::session_message::MessageContent::Text { text } => {
                            text.clone()
                        }
                        opencode_r_schema::session_message::MessageContent::ToolCall {
                            name, ..
                        } => format!("[tool:{}]", name),
                        opencode_r_schema::session_message::MessageContent::ToolResult {
                            content, ..
                        } => format!("[result: {}]", &content[..80.min(content.len())]),
                    })
                    .collect();
                xml.push_str(&format!("    <message role=\"{role}\">{text}</message>\n"));
            }
            xml.push_str("  </messages>\n</session>\n");
            xml
        }
        "md" => {
            let mut md = String::new();
            if let Some(s) = session {
                md.push_str(&format!("# {}\n\n", s.title));
                md.push_str(&format!(
                    "- **ID**: `{}`\n- **Status**: {}\n- **Model**: {}\n- **Cost**: ${:.4}\n",
                    s.id.0,
                    status_label(&s.status),
                    s.model.as_ref().map(|m| m.0.as_str()).unwrap_or("—"),
                    s.cost,
                ));
                md.push_str(&format!(
                    "- **Tokens**: {:.0} in / {:.0} out\n",
                    s.tokens.input, s.tokens.output
                ));
                md.push_str(&format!("- **Created**: {:?}\n\n---\n\n", s.time.created));
            }
            for m in messages {
                let role = match m.role {
                    opencode_r_schema::session_message::MessageRole::User => "**You**",
                    opencode_r_schema::session_message::MessageRole::Assistant => "**AI**",
                    opencode_r_schema::session_message::MessageRole::Tool => "**Tool**",
                    opencode_r_schema::session_message::MessageRole::System => "**System**",
                };
                md.push_str(&format!("### {role}\n\n"));
                for c in &m.content {
                    match c {
                        opencode_r_schema::session_message::MessageContent::Text { text } => {
                            md.push_str(text);
                            md.push_str("\n\n");
                        }
                        opencode_r_schema::session_message::MessageContent::ToolCall {
                            name,
                            arguments,
                            ..
                        } => {
                            md.push_str(&format!(
                                "```\n{name}({})\n```\n\n",
                                serde_json::to_string(arguments).unwrap_or_default()
                            ));
                        }
                        opencode_r_schema::session_message::MessageContent::ToolResult {
                            content, ..
                        } => {
                            md.push_str(&format!("```\n{}\n```\n\n", content));
                        }
                    }
                }
            }
            md
        }
        _ => String::new(),
    };

    std::fs::write(&filename, content).ok();
}

fn dirs_next() -> Option<std::path::PathBuf> {
    std::env::var("OPCODE_EXPORT_DIR")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| dirs_fallback().map(|d| d.join("opencode-exports")))
}

fn dirs_fallback() -> Option<std::path::PathBuf> {
    std::env::var("HOME").ok().map(std::path::PathBuf::from)
}
