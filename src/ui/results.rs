use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Sparkline},
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

    let has_chart = !app.wpm_samples.is_empty();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(1),                            // logo
            Constraint::Length(2),
            Constraint::Length(9),                              // stats (8 lines + pb)
            Constraint::Length(if has_chart { 1 } else { 0 }), // gap
            Constraint::Length(if has_chart { 4 } else { 0 }), // sparkline
            Constraint::Fill(1),
            Constraint::Length(1),                            // footer
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

        let mut lines = vec![
            stat_line("wpm", &wpm_s, theme),
            stat_line("raw", &raw_s, theme),
            stat_line("acc", &acc_s, theme),
            blank_line(theme),
            stat_line("chars", &chars_s, theme),
            stat_line("time", &time_s, theme),
            blank_line(theme),
            stat_line("mode", &result.mode, theme),
        ];

        // Personal best delta
        lines.push(pb_line(app.pb_delta, theme));

        f.render_widget(
            Paragraph::new(lines).style(bg_style).alignment(Alignment::Left),
            stat_area,
        );
    }

    // WPM sparkline
    if has_chart {
        let data: Vec<u64> = app.wpm_samples.iter().map(|&w| w as u64).collect();
        let max = data.iter().max().copied().unwrap_or(1) + 10;

        let chart_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(60),
                Constraint::Fill(1),
            ])
            .split(chunks[5])[1];

        let sparkline = Sparkline::default()
            .data(&data)
            .max(max)
            .style(Style::default().fg(theme.main.to_color()).bg(theme.bg.to_color()));

        f.render_widget(sparkline, chart_area);
    }

    footer(
        f,
        chunks[7],
        &[("enter/r", "retry"), ("esc", "home")],
        theme,
    );
}

fn pb_line(pb_delta: Option<f64>, theme: &crate::themes::Theme) -> Line<'static> {
    match pb_delta {
        None => {
            // First attempt — this run is the new PB
            Line::from(vec![
                Span::styled(
                    format!("{:<10}", "pb"),
                    Style::default().fg(theme.sub.to_color()),
                ),
                Span::styled(
                    "new pb!".to_string(),
                    Style::default()
                        .fg(theme.main.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        Some(delta) if delta >= 0.0 => {
            Line::from(vec![
                Span::styled(
                    format!("{:<10}", "pb"),
                    Style::default().fg(theme.sub.to_color()),
                ),
                Span::styled(
                    format!("+{:.1} wpm", delta),
                    Style::default()
                        .fg(theme.main.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        Some(delta) => {
            Line::from(vec![
                Span::styled(
                    format!("{:<10}", "pb"),
                    Style::default().fg(theme.sub.to_color()),
                ),
                Span::styled(
                    format!("{:.1} wpm", delta),
                    Style::default().fg(theme.sub.to_color()),
                ),
            ])
        }
    }
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
