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

    let full = f.area();
    f.render_widget(Paragraph::new("").style(bg_style), full);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),  // logo
            Constraint::Length(2),
            Constraint::Length(8),  // stats
            Constraint::Fill(1),
            Constraint::Length(1),  // footer
        ])
        .split(full);

    // Logo
    f.render_widget(
        header(theme).alignment(Alignment::Center).style(bg_style),
        chunks[1],
    );

    // Stats
    if let Some(result) = &app.last_result {
        let stat_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(40),
                Constraint::Fill(1),
            ])
            .split(chunks[3])[1];

        let wpm_s = format!("{:.0}", result.wpm);
        let raw_s = format!("{:.0}", result.raw_wpm);
        let acc_s = format!("{:.1}%", result.accuracy);
        let chars_s = format!("{} / {}", result.correct_chars, result.incorrect_chars);
        let time_s = format!("{:.1}s", result.duration.as_secs_f64());

        let lines = vec![
            stat_line("wpm", &wpm_s, theme),
            stat_line("raw", &raw_s, theme),
            stat_line("acc", &acc_s, theme),
            blank_line(theme),
            stat_line("chars", &chars_s, theme),
            stat_line("time", &time_s, theme),
            blank_line(theme),
            stat_line("mode", &result.mode, theme),
        ];

        f.render_widget(
            Paragraph::new(lines).style(bg_style).alignment(Alignment::Left),
            stat_area,
        );
    }

    footer(
        f,
        chunks[5],
        &[("enter/r", "retry"), ("esc", "home")],
        theme,
    );
}

fn stat_line<'a>(label: &'a str, value: &'a str, theme: &crate::themes::Theme) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{label:<10}"),
            Style::default().fg(theme.sub.to_color()),
        ),
        Span::styled(
            value.to_string(),
            Style::default()
                .fg(theme.main.to_color())
                .add_modifier(Modifier::BOLD),
        ),
    ])
}

fn blank_line(theme: &crate::themes::Theme) -> Line<'_> {
    Line::from(Span::styled(" ", Style::default().fg(theme.bg.to_color())))
}
