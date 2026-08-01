use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders};

/// Accent — cool ink cyan (not purple).
pub fn accent() -> Style {
    Style::default().fg(Color::Cyan)
}

pub fn title_style() -> Style {
    Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD)
}

pub fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}

pub fn ok() -> Style {
    Style::default().fg(Color::Green)
}

pub fn warn() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn highlight() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}

/// Shared framed panel — cyan border + titled label (matches hub header).
pub fn panel(title: &str) -> Block<'static> {
    let label = format!(" {title} ");
    Block::default()
        .borders(Borders::ALL)
        .border_style(accent())
        .title(Span::styled(label, accent()))
}
