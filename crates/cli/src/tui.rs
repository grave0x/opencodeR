// opencodeR TUI — terminal interface for server management
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    DefaultTerminal, Frame,
};
use chrono;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// A log entry from the server
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// Shared log buffer
pub struct LogBuffer {
    pub entries: Vec<LogEntry>,
    pub max_entries: usize,
}

impl LogBuffer {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            entries: Vec::with_capacity(1024),
            max_entries: 5000,
        }))
    }

    pub fn push(log: Arc<Mutex<Self>>, entry: LogEntry) {
        if let Ok(mut buf) = log.lock() {
            buf.entries.push(entry);
            if buf.entries.len() > buf.max_entries {
                buf.entries.remove(0);
            }
        }
    }
}

/// Tracing subscriber layer that feeds into the log buffer
pub struct TuiLogLayer {
    pub buffer: Arc<Mutex<LogBuffer>>,
}

impl<S> tracing_subscriber::Layer<S> for TuiLogLayer
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f").to_string();
        let meta = event.metadata();
        let level = meta.level().to_string();
        let target = meta.target().to_string();

        // Format the message
        let mut msg = String::new();
        let mut visitor = MsgVisitor(&mut msg);
        event.record(&mut visitor);

        LogBuffer::push(
            self.buffer.clone(),
            LogEntry {
                timestamp,
                level,
                target: target.to_string(),
                message: msg,
            },
        );
    }
}

struct MsgVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MsgVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            use std::fmt::Write;
            let _ = write!(self.0, "{:?}", value);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.0.push_str(value);
        }
    }
}

/// Run the TUI application
pub async fn run_tui(
    port: u16,
    password: Option<String>,
    log_buffer: Arc<Mutex<LogBuffer>>,
) -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    terminal.clear()?;

    let res = run_app(&mut terminal, port, password, log_buffer).await;

    ratatui::restore();
    res
}

async fn run_app(
    terminal: &mut DefaultTerminal,
    port: u16,
    password: Option<String>,
    log_buffer: Arc<Mutex<LogBuffer>>,
) -> anyhow::Result<()> {
    let mut scroll_offset: usize = 0;
    let server_running = Arc::new(Mutex::new(true));

    loop {
        terminal.draw(|frame| {
            draw_ui(frame, port, password.is_some(), &log_buffer, scroll_offset);
        })?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => {
                            let logbuf = log_buffer.lock().unwrap();
                            scroll_offset = scroll_offset.saturating_add(1).min(
                                logbuf.entries.len().saturating_sub(1),
                            );
                        }
                        KeyCode::Down => {
                            scroll_offset = scroll_offset.saturating_sub(1);
                        }
                        KeyCode::Char(' ') => {
                            // Toggle server running status
                            let mut running = server_running.lock().unwrap();
                            *running = !*running;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    Ok(())
}

fn draw_ui(
    frame: &mut Frame,
    port: u16,
    auth_enabled: bool,
    log_buffer: &Arc<Mutex<LogBuffer>>,
    scroll_offset: usize,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),   // Status bar
            Constraint::Min(0),       // Log panel
            Constraint::Length(3),   // Help bar
        ])
        .split(frame.area());

    // Status bar
    draw_status_bar(frame, chunks[0], port, auth_enabled);

    // Log panel
    draw_log_panel(frame, chunks[1], log_buffer, scroll_offset);

    // Help bar
    draw_help_bar(frame, chunks[2]);
}

fn draw_status_bar(frame: &mut Frame, area: Rect, port: u16, auth_enabled: bool) {
    let auth_status = if auth_enabled { "ON" } else { "OFF" };
    let text = vec![Line::from(vec![
        Span::styled(" opencodeR ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" | "),
        Span::styled("PORT", Style::default().fg(Color::Yellow)),
        Span::raw(format!(": {} ", port)),
        Span::styled("AUTH", Style::default().fg(Color::Yellow)),
        Span::styled(format!(": {} ", auth_status), 
            if auth_enabled { Style::default().fg(Color::Green) } else { Style::default().fg(Color::Red) }),
        Span::raw("| "),
        Span::styled("RUNNING", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ])];
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default());
    let paragraph = Paragraph::new(text).block(block);
    frame.render_widget(paragraph, area);
}

fn draw_log_panel(
    frame: &mut Frame,
    area: Rect,
    log_buffer: &Arc<Mutex<LogBuffer>>,
    scroll_offset: usize,
) {
    let logbuf = log_buffer.lock().unwrap();

    // Count per-level for the title
    let total = logbuf.entries.len();

    let items: Vec<ListItem> = if scroll_offset == 0 {
        // Auto-scroll: show last N items that fit
        let height = (area.height as usize).saturating_sub(2);
        let start = total.saturating_sub(height);
        logbuf.entries[start..]
            .iter()
            .rev()
            .map(|entry| format_log_entry(entry))
            .collect()
    } else {
        // Manual scroll
        let end = total.saturating_sub(scroll_offset);
        let start = end.saturating_sub((area.height as usize).saturating_sub(2));
        logbuf.entries[start..end]
            .iter()
            .rev()
            .map(|entry| format_log_entry(entry))
            .collect()
    };

    let title = format!(" Server Log ({} events) ", total);
    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(Color::Cyan))
        .borders(Borders::ALL)
        .border_set(border::ROUNDED);

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn format_log_entry(entry: &LogEntry) -> ListItem<'static> {
    let (color, prefix) = match entry.level.as_str() {
        "ERROR" => (Color::Red, "ERR"),
        "WARN" => (Color::Yellow, "WRN"),
        "INFO" => (Color::Green, "INF"),
        "DEBUG" => (Color::Blue, "DBG"),
        "TRACE" => (Color::DarkGray, "TRC"),
        _ => (Color::White, "???"),
    };

    // Truncate message for display
    let display_msg = if entry.message.len() > 120 {
        format!("{}...", &entry.message[..117])
    } else {
        entry.message.clone()
    };

    let line = Line::from(vec![
        Span::styled(format!(" {} ", entry.timestamp), Style::default().fg(Color::DarkGray)),
        Span::styled(format!(" {} ", prefix), Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(display_msg, Style::default()),
    ]);
    ListItem::new(line)
}

fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let text = Line::from(vec![
        Span::styled(" ↑↓ ", Style::default().fg(Color::DarkGray)),
        Span::raw("Scroll  "),
        Span::styled(" Q/Esc ", Style::default().fg(Color::DarkGray)),
        Span::raw("Quit  "),
        Span::styled(" SPACE ", Style::default().fg(Color::DarkGray)),
        Span::raw("Toggle"),
    ]);
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::DarkGray));
    let paragraph = Paragraph::new(text).block(block).centered();
    frame.render_widget(paragraph, area);
}
