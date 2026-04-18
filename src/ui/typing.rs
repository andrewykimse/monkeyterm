use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use unicode_width::UnicodeWidthChar;

use crate::app::{App, CharState, TestMode};
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
            Constraint::Fill(1),
            Constraint::Length(1),  // stats bar
            Constraint::Length(1),
            Constraint::Length(5),  // typing area
            Constraint::Fill(1),
            Constraint::Length(1),  // footer
        ])
        .split(full);

    // Stats bar
    draw_stats(f, app, chunks[1]);

    // Typing area (centered horizontally)
    let type_area = centered_rect(80, chunks[3]);
    draw_words(f, app, type_area);

    // Footer
    footer(
        f,
        chunks[5],
        &[("tab", "restart"), ("esc", "home")],
        theme,
    );
}

fn draw_stats(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let wpm_text = if app.test_started {
        format!("{:.0} wpm", app.live_wpm)
    } else {
        "-- wpm".into()
    };

    let acc_text = if app.test_started {
        format!("{:.1}%", app.live_accuracy)
    } else {
        "--%".into()
    };

    let timer_text = match &app.mode {
        TestMode::Time(_) => {
            if let Some(rem) = app.time_remaining {
                format!("{:.0}s", rem.ceil())
            } else if let TestMode::Time(secs) = app.mode {
                format!("{secs}s")
            } else {
                String::new()
            }
        }
        TestMode::Words(n) => {
            let done = app.current_word;
            format!("{done}/{n}")
        }
        TestMode::Quote => {
            let done = app.current_word;
            let total = app.words.len();
            format!("{done}/{total}")
        }
        TestMode::Zen => String::new(),
    };

    let spans = vec![
        Span::styled(
            format!(" {wpm_text}"),
            Style::default()
                .fg(theme.main.to_color())
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {acc_text}"),
            Style::default().fg(theme.sub.to_color()),
        ),
        Span::styled(
            format!("  {timer_text}"),
            Style::default().fg(theme.sub.to_color()),
        ),
    ];

    f.render_widget(
        Paragraph::new(Line::from(spans))
            .style(Style::default().bg(theme.bg.to_color()))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_words(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let width = area.width as usize;
    let current_word_idx = app.current_word;

    // First pass: lay out every word into lines and record which line each word lands on.
    let mut all_lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current_spans: Vec<Span<'static>> = Vec::new();
    let mut current_width = 0usize;
    let mut word_to_line: Vec<usize> = Vec::with_capacity(app.words.len());

    for (word_idx, word) in app.words.iter().enumerate() {
        let is_current = word_idx == current_word_idx;
        let word_display_width: usize = word
            .expected
            .iter()
            .map(|c| c.width().unwrap_or(1))
            .sum::<usize>()
            .max(word.typed.len());
        let needed = word_display_width + if current_width > 0 { 1 } else { 0 };

        if current_width > 0 && current_width + needed > width {
            all_lines.push(std::mem::take(&mut current_spans));
            current_width = 0;
        }

        word_to_line.push(all_lines.len());

        if current_width > 0 {
            current_spans.push(Span::styled(
                " ".to_string(),
                Style::default().fg(theme.sub.to_color()).bg(theme.bg.to_color()),
            ));
            current_width += 1;
        }

        current_spans.extend(build_word_spans(word, is_current, app));
        current_width += word_display_width;
    }
    if !current_spans.is_empty() {
        all_lines.push(current_spans);
    }

    // Find which line the current word is on and start rendering from there.
    let start_line = word_to_line
        .get(current_word_idx)
        .copied()
        .unwrap_or(all_lines.len().saturating_sub(1));

    let visible: Vec<Line> = all_lines
        .into_iter()
        .skip(start_line)
        .take(3)
        .map(Line::from)
        .collect();

    let bg_style = Style::default().bg(theme.bg.to_color()).fg(theme.fg.to_color());
    f.render_widget(
        Paragraph::new(visible).style(bg_style),
        area,
    );
}

fn build_word_spans(
    word: &crate::app::WordState,
    is_current: bool,
    app: &App,
) -> Vec<Span<'static>> {
    let theme = &app.theme;
    let mut spans = Vec::new();

    let max_len = word.expected.len().max(word.typed.len());

    for i in 0..max_len {
        let ch = if i < word.expected.len() {
            word.expected[i]
        } else {
            // extra typed character
            word.typed[i]
        };

        let state = word.char_state(i);

        let style = match state {
            CharState::Untyped => {
                let mut s = Style::default()
                    .fg(theme.sub.to_color())
                    .bg(theme.bg.to_color());
                // Caret: highlight the first untyped char of current word
                if is_current && i == word.typed.len() {
                    s = s.bg(theme.caret.to_color()).fg(theme.bg.to_color());
                }
                s
            }
            CharState::Correct => Style::default()
                .fg(theme.fg.to_color())
                .bg(theme.bg.to_color()),
            CharState::Incorrect => Style::default()
                .fg(theme.error.to_color())
                .bg(theme.bg.to_color()),
            CharState::Extra => Style::default()
                .fg(theme.error.to_color())
                .add_modifier(Modifier::UNDERLINED)
                .bg(theme.bg.to_color()),
        };

        spans.push(Span::styled(ch.to_string(), style));
    }

    // If current word has no typed chars yet, ensure caret on first char
    if is_current && word.typed.is_empty() && !word.expected.is_empty() {
        if let Some(first_span) = spans.first_mut() {
            *first_span = Span::styled(
                word.expected[0].to_string(),
                Style::default()
                    .bg(theme.caret.to_color())
                    .fg(theme.bg.to_color()),
            );
        }
    }

    spans
}

fn centered_rect(percent_x: u16, r: Rect) -> Rect {
    let w = r.width * percent_x / 100;
    let x = r.x + (r.width - w) / 2;
    Rect {
        x,
        y: r.y,
        width: w,
        height: r.height,
    }
}
