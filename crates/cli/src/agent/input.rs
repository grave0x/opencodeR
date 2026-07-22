use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::agent::AgentState;

pub fn draw(frame: &mut Frame, area: Rect, state: &AgentState) {
    let display = if state.input.is_empty() {
        " Type message here. /quit to exit.".to_string()
    } else {
        format!(" {}", state.input)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    let para = Paragraph::new(display)
        .block(block)
        .style(Style::default().fg(if state.input.is_empty() { Color::DarkGray } else { Color::White }));
    frame.render_widget(para, area);
}
