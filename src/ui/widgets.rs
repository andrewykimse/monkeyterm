use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::themes::Theme;

/// Renders a centered title block with the monkeyterm branding.
pub fn header<'a>(theme: &Theme) -> Paragraph<'a> {
    let title = Span::styled(
        "monkeyterm",
        Style::default()
            .fg(theme.main.to_color())
            .add_modifier(Modifier::BOLD),
    );
    Paragraph::new(Line::from(vec![title]))
}

/// Simple centered block outline
pub fn bordered_block<'a>(title: &'a str, theme: &Theme) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.sub.to_color()))
        .title(Span::styled(
            title,
            Style::default().fg(theme.main.to_color()),
        ))
}

/// Draw a small footer hint bar
pub fn footer(f: &mut Frame, area: Rect, hints: &[(&str, &str)], theme: &Theme) {
    let mut spans: Vec<Span> = Vec::new();
    for (i, (key, desc)) in hints.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        spans.push(Span::styled(
            format!("[{key}]"),
            Style::default().fg(theme.main.to_color()),
        ));
        spans.push(Span::styled(
            format!(" {desc}"),
            Style::default().fg(theme.sub.to_color()),
        ));
    }
    let p = Paragraph::new(Line::from(spans));
    f.render_widget(p, area);
}
