use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::themes::Theme;
use super::widgets::footer;

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let bg_style = Style::default()
        .bg(theme.bg.to_color())
        .fg(theme.fg.to_color());

    let full = f.area();
    f.render_widget(Paragraph::new("").style(bg_style), full);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(4),  // preview swatches
            Constraint::Length(1),  // footer
        ])
        .split(full);

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled(
            "themes",
            Style::default()
                .fg(theme.main.to_color())
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .alignment(Alignment::Center)
    .style(bg_style);
    f.render_widget(title, chunks[0]);

    // Theme list (two-column layout)
    let all_themes = Theme::all();
    let selected = app.theme_picker_selected;

    let list_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(40),
            Constraint::Fill(1),
        ])
        .split(chunks[1])[1];

    let lines: Vec<Line> = all_themes
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let is_selected = i == selected;
            let prefix = if is_selected { "> " } else { "  " };
            let style = if is_selected {
                Style::default()
                    .fg(theme.main.to_color())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.sub.to_color())
            };
            Line::from(Span::styled(format!("{prefix}{}", t.name), style))
        })
        .collect();

    f.render_widget(
        Paragraph::new(lines).style(bg_style),
        list_area,
    );

    // Color swatches preview for selected theme
    let preview_theme = &all_themes[selected];
    draw_swatches(f, preview_theme, chunks[2]);

    footer(
        f,
        chunks[3],
        &[("↑↓/jk", "navigate"), ("enter", "select"), ("esc", "back")],
        theme,
    );
}

fn draw_swatches(f: &mut Frame, theme: &Theme, area: ratatui::layout::Rect) {
    let swatch_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Fill(1),
        ])
        .split(area);

    let colors = [
        ("bg", theme.bg.to_color()),
        ("fg", theme.fg.to_color()),
        ("main", theme.main.to_color()),
        ("sub", theme.sub.to_color()),
        ("err", theme.error.to_color()),
        ("cur", theme.caret.to_color()),
    ];

    for (i, (label, color)) in colors.iter().enumerate() {
        let chunk = swatch_chunks[i + 1];
        let label_area = ratatui::layout::Rect {
            x: chunk.x,
            y: chunk.y + chunk.height.saturating_sub(1),
            width: chunk.width,
            height: 1,
        };
        let swatch_area = ratatui::layout::Rect {
            x: chunk.x,
            y: chunk.y,
            width: chunk.width,
            height: chunk.height.saturating_sub(1),
        };

        // Color block
        f.render_widget(
            Paragraph::new("").style(Style::default().bg(*color)),
            swatch_area,
        );

        // Label below
        f.render_widget(
            Paragraph::new(Line::from(Span::styled(
                label.to_string(),
                Style::default()
                    .fg(theme.sub.to_color())
                    .bg(theme.bg.to_color()),
            )))
            .alignment(Alignment::Center),
            label_area,
        );
    }
}
