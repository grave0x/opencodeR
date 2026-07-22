use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem};
use ratatui::Frame;

use crate::agent::AgentState;

pub fn draw(frame: &mut Frame, area: Rect, state: &AgentState) {
    let items: Vec<ListItem> = if state.scroll_offset == 0 {
        // Auto-scroll: show last N that fit
        let height = (area.height as usize).saturating_sub(2);
        let start = state.messages.len().saturating_sub(height);
        state.messages[start..].iter().map(|m| format_message(m)).collect()
    } else {
        let end = state.messages.len().saturating_sub(state.scroll_offset);
        let start = end.saturating_sub((area.height as usize).saturating_sub(2));
        if start >= end {
            vec![]
        } else {
            state.messages[start..end].iter().map(|m| format_message(m)).collect()
        }
    };

    let title = format!(" Conversation ({} messages) ", state.messages.len());
    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn format_message(msg: &opencode_r_schema::session_message::SessionMessage) -> ListItem<'static> {
    let (label, color) = match msg.role {
        opencode_r_schema::session_message::MessageRole::User => (" YOU ", Color::Green),
        opencode_r_schema::session_message::MessageRole::Assistant => (" AI  ", Color::Cyan),
        opencode_r_schema::session_message::MessageRole::Tool => (" TOOL", Color::Yellow),
        opencode_r_schema::session_message::MessageRole::System => (" SYS ", Color::Magenta),
    };

    let text: String = msg.content.iter().map(|c| match c {
        opencode_r_schema::session_message::MessageContent::Text { text } => text.clone(),
        opencode_r_schema::session_message::MessageContent::ToolCall { name, .. } =>
            format!("[tool_call: {}]", name),
        opencode_r_schema::session_message::MessageContent::ToolResult { content, .. } =>
            format!("[result: {}]", &content[..80.min(content.len())]),
    }).collect::<Vec<_>>().join("\n");

    let preview = if text.len() > 500 {
        format!("{}...", &text[..497])
    } else {
        text
    };

    let line = Line::from(vec![
        Span::styled(label, Style::default().fg(color).add_modifier(Modifier::BOLD)),
        Span::raw(" "),
        Span::styled(preview, Style::default()),
    ]);
    ListItem::new(line)
}
