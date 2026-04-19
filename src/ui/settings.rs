use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use super::widgets::{footer, header};

const SETTINGS: &[(&str, &str)] = &[
    ("theme", "cycle through available themes"),
    ("word list", "toggle english / code"),
    ("default mode", "cycle through test modes"),
];

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
            Constraint::Fill(1),
            Constraint::Length(1),  // logo
            Constraint::Length(2),
            Constraint::Length(9),  // settings rows
            Constraint::Fill(1),
            Constraint::Length(1),  // footer
        ])
        .split(full);

    f.render_widget(
        header(theme).alignment(Alignment::Center).style(bg_style),
        chunks[1],
    );

    let title = Paragraph::new(Span::styled(
        "settings",
        Style::default().fg(theme.sub.to_color()),
    ))
    .alignment(Alignment::Center)
    .style(bg_style);
    f.render_widget(title, chunks[2]);

    let settings_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(36),
            Constraint::Fill(1),
        ])
        .split(chunks[3])[1];

    let current_values = [
        app.config.theme.clone(),
        app.config.word_list.clone(),
        app.config.default_mode.clone(),
    ];

    let mut lines: Vec<Line> = Vec::new();
    for (i, ((label, _hint), value)) in SETTINGS.iter().zip(current_values.iter()).enumerate() {
        let is_selected = i == app.settings_selected;

        let label_style = if is_selected {
            Style::default()
                .fg(theme.bg.to_color())
                .bg(theme.main.to_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.sub.to_color())
        };
        let value_style = if is_selected {
            Style::default()
                .fg(theme.bg.to_color())
                .bg(theme.main.to_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(theme.main.to_color())
                .add_modifier(Modifier::BOLD)
        };

        let row = Line::from(vec![
            Span::styled(format!("{label:<16}"), label_style),
            Span::styled(value.clone(), value_style),
        ]);
        lines.push(row);

        if i < SETTINGS.len() - 1 {
            lines.push(Line::from(Span::styled(
                " ",
                Style::default().fg(theme.bg.to_color()),
            )));
        }
    }

    f.render_widget(
        Paragraph::new(lines).style(bg_style).alignment(Alignment::Left),
        settings_area,
    );

    footer(
        f,
        chunks[5],
        &[("↑/↓", "navigate"), ("enter", "cycle"), ("esc", "back")],
        theme,
    );
}
