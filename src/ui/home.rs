use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use super::widgets::{footer, header};

pub fn draw(f: &mut Frame, app: &App) {
    let theme = &app.theme;
    let bg_style = Style::default()
        .bg(theme.bg.to_color())
        .fg(theme.fg.to_color());

    // Fill background
    let full = f.area();
    f.render_widget(
        Paragraph::new("").style(bg_style),
        full,
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(3),  // logo
            Constraint::Length(1),
            Constraint::Length(9),  // menu
            Constraint::Fill(1),
            Constraint::Length(1),  // footer
        ])
        .split(full);

    // Logo
    let logo = header(theme).alignment(Alignment::Center).style(bg_style);
    f.render_widget(logo, chunks[1]);

    // Subtitle
    let sub = Paragraph::new(Span::styled(
        "a terminal typing test",
        Style::default().fg(theme.sub.to_color()),
    ))
    .alignment(Alignment::Center)
    .style(bg_style);
    f.render_widget(sub, chunks[2]);

    // Menu
    let menu_items: Vec<Line> = vec![
        make_menu_line("1", "words  25", theme),
        make_menu_line("2", "words  50", theme),
        make_menu_line("3", "words 100", theme),
        blank_line(theme),
        make_menu_line("4", "time   30s", theme),
        make_menu_line("5", "time   60s", theme),
        make_menu_line("6", "time  120s", theme),
        blank_line(theme),
        make_menu_line("c", "quote", theme),
    ];

    let menu_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(22),
            Constraint::Fill(1),
        ])
        .split(chunks[3])[1];

    let menu = Paragraph::new(menu_items)
        .style(bg_style)
        .alignment(Alignment::Left);
    f.render_widget(menu, menu_area);

    // Footer
    footer(
        f,
        chunks[5],
        &[("t", "theme"), ("q", "quit")],
        theme,
    );
}

fn make_menu_line<'a>(key: &'a str, label: &'a str, theme: &crate::themes::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!(" [{key}] "),
            Style::default()
                .fg(theme.main.to_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(label, Style::default().fg(theme.fg.to_color())),
    ])
}

fn blank_line(theme: &crate::themes::Theme) -> Line<'_> {
    Line::from(Span::styled(" ", Style::default().fg(theme.bg.to_color())))
}
