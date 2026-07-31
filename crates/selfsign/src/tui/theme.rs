use ratatui::style::{Color, Modifier, Style};

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

pub fn highlight() -> Style {
    Style::default()
        .fg(Color::Black)
        .bg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
}
