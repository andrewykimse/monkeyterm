use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::history::History;
use crate::themes::Theme;
use crate::words::{WordList, generate_word_list, get_quote};

// ---------------------------------------------------------------------------
// Test modes
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum TestMode {
    /// Fixed number of words
    Words(usize),
    /// Countdown timer in seconds
    Time(u64),
    /// Type a specific quote
    Quote,
    /// Zen: no stats, no end
    Zen,
}

impl TestMode {
    pub fn label(&self) -> String {
        match self {
            TestMode::Words(n) => format!("words {n}"),
            TestMode::Time(s) => format!("time {s}s"),
            TestMode::Quote => "quote".into(),
            TestMode::Zen => "zen".into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Per-character state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CharState {
    Untyped,
    Correct,
    Incorrect,
    Extra, // typed character beyond word length
}

#[derive(Debug, Clone)]
pub struct WordState {
    pub expected: Vec<char>,
    pub typed: Vec<char>,
}

impl WordState {
    pub fn new(word: &str) -> Self {
        Self {
            expected: word.chars().collect(),
            typed: Vec::new(),
        }
    }

    pub fn char_state(&self, idx: usize) -> CharState {
        match self.typed.get(idx) {
            None => CharState::Untyped,
            Some(c) if self.expected.get(idx) == Some(c) => CharState::Correct,
            Some(_) => CharState::Incorrect,
        }
    }

    pub fn is_complete(&self) -> bool {
        !self.typed.is_empty() && self.typed.len() >= self.expected.len()
    }

    pub fn is_correct(&self) -> bool {
        self.typed == self.expected
    }
}

// ---------------------------------------------------------------------------
// Test results
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TestResult {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub duration: Duration,
    pub mode: String,
}

// ---------------------------------------------------------------------------
// Screen / app state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Screen {
    Home,
    Typing,
    Results,
    ThemePicker,
    Settings,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    // Navigation
    pub screen: Screen,
    pub should_quit: bool,

    // Persisted config
    pub config: Config,

    // Theme
    pub theme: Theme,
    pub theme_index: usize,

    // Test configuration
    pub mode: TestMode,
    pub word_list: WordList,

    // Typing state
    pub words: Vec<WordState>,
    pub current_word: usize,
    pub current_input: String, // buffer for the word being typed
    pub test_started: bool,
    pub test_finished: bool,
    pub start_time: Option<Instant>,
    pub time_limit: Option<Duration>,
    pub time_remaining: Option<f64>, // seconds

    // Live stats
    pub live_wpm: f64,
    pub live_accuracy: f64,

    // Results
    pub last_result: Option<TestResult>,
    /// Delta vs personal best at time of last result (None = first attempt).
    pub pb_delta: Option<f64>,

    // Theme picker state
    pub theme_picker_selected: usize,

    // History
    pub history: History,

    // WPM over time (one sample per elapsed second)
    pub wpm_samples: Vec<f64>,
    pub last_sample_second: u64,

    // Settings screen state
    pub settings_selected: usize,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load();

        // Resolve theme from config (fall back to index 0 if not found)
        let themes = Theme::all();
        let theme_index = themes
            .iter()
            .position(|t| t.name == config.theme)
            .unwrap_or(0);
        let theme = themes[theme_index].clone();

        let word_list = config.word_list();
        let mode = config.test_mode();

        let word_count = match &mode {
            TestMode::Words(n) => *n,
            TestMode::Time(_) => 200,
            TestMode::Quote | TestMode::Zen => 0,
        };
        let words = if matches!(mode, TestMode::Quote) {
            let quote = get_quote();
            quote.split_whitespace().map(WordState::new).collect()
        } else {
            generate_word_list(&word_list, word_count)
                .iter()
                .map(|w| WordState::new(w))
                .collect()
        };

        Ok(Self {
            screen: Screen::Home,
            should_quit: false,
            config,
            theme,
            theme_index,
            mode,
            word_list,
            words,
            current_word: 0,
            current_input: String::new(),
            test_started: false,
            test_finished: false,
            start_time: None,
            time_limit: None,
            time_remaining: None,
            live_wpm: 0.0,
            live_accuracy: 0.0,
            last_result: None,
            pb_delta: None,
            theme_picker_selected: 0,
            history: History::load(),
            wpm_samples: Vec::new(),
            last_sample_second: 0,
            settings_selected: 0,
        })
    }

    // -----------------------------------------------------------------------
    // Input handling
    // -----------------------------------------------------------------------

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::Typing => self.handle_typing_key(key),
            Screen::Results => self.handle_results_key(key),
            Screen::ThemePicker => self.handle_theme_picker_key(key),
            Screen::Settings => self.handle_settings_key(key),
        }
        Ok(())
    }

    fn handle_home_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => self.start_test(),
            KeyCode::Char('t') => {
                self.screen = Screen::ThemePicker;
                self.theme_picker_selected = self.theme_index;
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('w') => {
                self.word_list = match self.word_list {
                    WordList::Common200 => WordList::Programming,
                    WordList::Programming => WordList::Common200,
                };
                self.config.word_list = Config::word_list_from(&self.word_list);
                let _ = self.config.save();
            }
            KeyCode::Char('1') => {
                self.mode = TestMode::Words(25);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('2') => {
                self.mode = TestMode::Words(50);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('3') => {
                self.mode = TestMode::Words(100);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('4') => {
                self.mode = TestMode::Time(30);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('5') => {
                self.mode = TestMode::Time(60);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('6') => {
                self.mode = TestMode::Time(120);
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('c') => {
                self.mode = TestMode::Quote;
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('z') => {
                self.mode = TestMode::Zen;
                self.config.default_mode = Config::test_mode_from(&self.mode);
                let _ = self.config.save();
                self.start_test();
            }
            KeyCode::Char('s') => {
                self.screen = Screen::Settings;
            }
            _ => {}
        }
    }

    fn handle_typing_key(&mut self, key: KeyEvent) {
        if self.test_finished {
            match key.code {
                KeyCode::Enter | KeyCode::Char(' ') => self.screen = Screen::Results,
                KeyCode::Esc | KeyCode::Tab => self.restart_test(),
                _ => {}
            }
            return;
        }

        match key.code {
            // Escape / Tab: restart
            KeyCode::Esc | KeyCode::Tab => {
                self.screen = Screen::Home;
                self.reset_test();
            }

            // Backspace: delete last char from current input
            KeyCode::Backspace => {
                if key.modifiers.contains(KeyModifiers::CONTROL) {
                    // Ctrl+Backspace: delete whole word
                    self.current_input.clear();
                } else if !self.current_input.is_empty() {
                    self.current_input.pop();
                } else if self.current_word > 0 {
                    // go back to previous word
                    self.current_word -= 1;
                    self.current_input =
                        self.words[self.current_word].typed.iter().collect();
                    self.words[self.current_word].typed.clear();
                }
                self.sync_current_word_typed();
            }

            // Space: commit word
            KeyCode::Char(' ') => {
                if !self.current_input.is_empty() {
                    self.commit_word();
                }
            }

            // Regular character input
            KeyCode::Char(c) => {
                if !self.test_started {
                    self.test_started = true;
                    self.start_time = Some(Instant::now());
                    if let TestMode::Time(secs) = self.mode {
                        self.time_limit = Some(Duration::from_secs(secs));
                        self.time_remaining = Some(secs as f64);
                    }
                }
                self.current_input.push(c);
                self.sync_current_word_typed();

                // Complete the test when the last word is fully typed (no trailing space needed)
                if let TestMode::Words(n) = self.mode {
                    if self.current_word == n - 1 {
                        let word = &self.words[self.current_word];
                        if word.typed.len() >= word.expected.len() {
                            self.current_word += 1;
                            self.current_input.clear();
                            self.finish_test();
                        }
                    }
                }
            }

            _ => {}
        }
    }

    fn handle_results_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Enter | KeyCode::Char('r') | KeyCode::Tab => self.restart_test(),
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Home,
            _ => {}
        }
    }

    fn handle_theme_picker_key(&mut self, key: KeyEvent) {
        let count = Theme::all().len();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Home,
            KeyCode::Enter => {
                self.theme_index = self.theme_picker_selected;
                self.theme = Theme::all()[self.theme_index].clone();
                self.config.theme = self.theme.name.clone();
                let _ = self.config.save();
                self.screen = Screen::Home;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.theme_picker_selected > 0 {
                    self.theme_picker_selected -= 1;
                } else {
                    self.theme_picker_selected = count - 1;
                }
                // Preview theme
                self.theme = Theme::all()[self.theme_picker_selected].clone();
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.theme_picker_selected = (self.theme_picker_selected + 1) % count;
                self.theme = Theme::all()[self.theme_picker_selected].clone();
            }
            _ => {}
        }
    }

    fn handle_settings_key(&mut self, key: KeyEvent) {
        const NUM_SETTINGS: usize = 3;
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.screen = Screen::Home,
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_selected > 0 {
                    self.settings_selected -= 1;
                } else {
                    self.settings_selected = NUM_SETTINGS - 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.settings_selected = (self.settings_selected + 1) % NUM_SETTINGS;
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
                match self.settings_selected {
                    0 => {
                        // Cycle theme
                        let themes = Theme::all();
                        self.theme_index = (self.theme_index + 1) % themes.len();
                        self.theme = themes[self.theme_index].clone();
                        self.config.theme = self.theme.name.clone();
                        let _ = self.config.save();
                    }
                    1 => {
                        // Toggle word list
                        self.word_list = match self.word_list {
                            WordList::Common200 => WordList::Programming,
                            WordList::Programming => WordList::Common200,
                        };
                        self.config.word_list = Config::word_list_from(&self.word_list);
                        let _ = self.config.save();
                    }
                    2 => {
                        // Cycle default mode
                        let modes = [
                            "words:25", "words:50", "words:100",
                            "time:30", "time:60", "time:120", "quote",
                        ];
                        let current = self.config.default_mode.as_str();
                        let idx = modes.iter().position(|&m| m == current).unwrap_or(0);
                        let next = modes[(idx + 1) % modes.len()];
                        self.config.default_mode = next.to_string();
                        self.mode = self.config.test_mode();
                        let _ = self.config.save();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Test lifecycle
    // -----------------------------------------------------------------------

    pub fn start_test(&mut self) {
        self.reset_test();
        self.screen = Screen::Typing;
    }

    pub fn restart_test(&mut self) {
        self.reset_test();
        self.screen = Screen::Typing;
    }

    fn reset_test(&mut self) {
        self.test_started = false;
        self.test_finished = false;
        self.start_time = None;
        self.time_limit = None;
        self.time_remaining = None;
        self.current_word = 0;
        self.current_input.clear();
        self.live_wpm = 0.0;
        self.live_accuracy = 0.0;
        self.wpm_samples.clear();
        self.last_sample_second = 0;

        let count = match &self.mode {
            TestMode::Words(n) => *n,
            TestMode::Time(_) => 200, // generate plenty
            TestMode::Zen => 100,     // initial batch; auto-expands
            TestMode::Quote => 0,
        };

        self.words = if matches!(self.mode, TestMode::Quote) {
            let quote = get_quote();
            quote
                .split_whitespace()
                .map(WordState::new)
                .collect()
        } else {
            generate_word_list(&self.word_list, count)
                .iter()
                .map(|w| WordState::new(w))
                .collect()
        };
    }

    fn commit_word(&mut self) {
        // Store typed chars into word state
        self.words[self.current_word].typed = self.current_input.chars().collect();
        self.current_input.clear();

        self.current_word += 1;

        // Check if we've finished all words (words mode)
        if let TestMode::Words(n) = self.mode {
            if self.current_word >= n {
                self.finish_test();
            }
        }

        // Auto-expand for time and zen modes
        if matches!(self.mode, TestMode::Time(_) | TestMode::Zen)
            && self.current_word >= self.words.len() - 20
        {
            let extra = generate_word_list(&self.word_list, 50);
            self.words
                .extend(extra.iter().map(|w| WordState::new(w)));
        }
    }

    fn sync_current_word_typed(&mut self) {
        if let Some(word) = self.words.get_mut(self.current_word) {
            word.typed = self.current_input.chars().collect();
        }
    }

    fn finish_test(&mut self) {
        self.test_finished = true;
        let duration = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or(Duration::from_secs(1));

        let (correct, incorrect) = self.count_chars();
        let minutes = duration.as_secs_f64() / 60.0;
        let raw_wpm = (correct + incorrect) as f64 / 5.0 / minutes;
        let wpm = correct as f64 / 5.0 / minutes;
        let accuracy = if correct + incorrect > 0 {
            correct as f64 / (correct + incorrect) as f64 * 100.0
        } else {
            100.0
        };

        let mode_label = self.mode.label();
        // Snapshot PB before adding new result so delta is vs previous best
        let prev_best = self.history.personal_best(&mode_label).map(|e| e.wpm);

        // Final WPM sample
        self.wpm_samples.push(wpm);

        let result = TestResult {
            wpm,
            raw_wpm,
            accuracy,
            correct_chars: correct,
            incorrect_chars: incorrect,
            duration,
            mode: mode_label,
        };

        self.pb_delta = prev_best.map(|pb| result.wpm - pb);
        self.history.add(&result);
        let _ = self.history.save();
        self.last_result = Some(result);
    }

    fn count_chars(&self) -> (usize, usize) {
        count_chars_in(&self.words, self.current_word)
    }
}

// ---------------------------------------------------------------------------
// Pure helper functions (extracted for testability)
// ---------------------------------------------------------------------------

/// Count correct and incorrect keystrokes across `words[0..up_to]`.
pub fn count_chars_in(words: &[WordState], up_to: usize) -> (usize, usize) {
    let mut correct = 0;
    let mut incorrect = 0;
    for (i, word) in words.iter().enumerate() {
        if i >= up_to {
            break;
        }
        for (j, typed) in word.typed.iter().enumerate() {
            if word.expected.get(j) == Some(typed) {
                correct += 1;
            } else {
                // Covers both mistyped chars and extra chars beyond word length
                // (expected.get(j) returns None for j >= expected.len())
                incorrect += 1;
            }
        }
    }
    (correct, incorrect)
}

/// Compute (wpm, raw_wpm, accuracy) from raw counts and elapsed seconds.
pub fn calculate_stats(correct: usize, incorrect: usize, duration_secs: f64) -> (f64, f64, f64) {
    let minutes = duration_secs / 60.0;
    let raw_wpm = (correct + incorrect) as f64 / 5.0 / minutes;
    let wpm = correct as f64 / 5.0 / minutes;
    let accuracy = if correct + incorrect > 0 {
        correct as f64 / (correct + incorrect) as f64 * 100.0
    } else {
        100.0
    };
    (wpm, raw_wpm, accuracy)
}

// Keep `impl App` open for the tick method below
impl App {

    // -----------------------------------------------------------------------
    // Tick (called each frame)
    // -----------------------------------------------------------------------

    pub fn tick(&mut self) {
        if !self.test_started || self.test_finished {
            return;
        }

        let elapsed = self
            .start_time
            .map(|t| t.elapsed())
            .unwrap_or_default();

        // Update timer for time mode
        if let Some(limit) = self.time_limit {
            let remaining = limit.as_secs_f64() - elapsed.as_secs_f64();
            if remaining <= 0.0 {
                self.time_remaining = Some(0.0);
                self.finish_test();
                self.screen = Screen::Results;
                return;
            }
            self.time_remaining = Some(remaining);
        }

        // Update live WPM
        let minutes = elapsed.as_secs_f64() / 60.0;
        if minutes > 0.0 {
            let (correct, incorrect) = self.count_chars();
            self.live_wpm = correct as f64 / 5.0 / minutes;
            let total = correct + incorrect;
            self.live_accuracy = if total > 0 {
                correct as f64 / total as f64 * 100.0
            } else {
                100.0
            };

            // Push a WPM sample once per elapsed second
            let current_second = elapsed.as_secs();
            if current_second > self.last_sample_second {
                self.wpm_samples.push(self.live_wpm);
                self.last_sample_second = current_second;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- WordState ----

    #[test]
    fn word_state_untyped_chars_are_untyped() {
        let w = WordState::new("hello");
        assert_eq!(w.char_state(0), CharState::Untyped);
        assert_eq!(w.char_state(4), CharState::Untyped);
    }

    #[test]
    fn word_state_correct_chars() {
        let mut w = WordState::new("hello");
        w.typed = "hel".chars().collect();
        assert_eq!(w.char_state(0), CharState::Correct);
        assert_eq!(w.char_state(2), CharState::Correct);
        assert_eq!(w.char_state(3), CharState::Untyped);
    }

    #[test]
    fn word_state_incorrect_char() {
        let mut w = WordState::new("hello");
        w.typed = "helXo".chars().collect();
        assert_eq!(w.char_state(3), CharState::Incorrect);
        assert_eq!(w.char_state(0), CharState::Correct);
    }

    #[test]
    fn word_state_is_correct_when_fully_typed() {
        let mut w = WordState::new("hello");
        w.typed = "hello".chars().collect();
        assert!(w.is_correct());
        assert!(w.is_complete());
    }

    #[test]
    fn word_state_is_not_correct_with_typo() {
        let mut w = WordState::new("hello");
        w.typed = "hellx".chars().collect();
        assert!(!w.is_correct());
        assert!(w.is_complete());
    }

    #[test]
    fn word_state_not_complete_when_empty() {
        let w = WordState::new("hello");
        assert!(!w.is_complete());
        assert!(!w.is_correct());
    }

    #[test]
    fn word_state_not_complete_when_partial() {
        let mut w = WordState::new("hello");
        w.typed = "hel".chars().collect();
        assert!(!w.is_complete());
    }

    // ---- count_chars_in ----

    #[test]
    fn count_chars_all_correct() {
        let mut words = vec![WordState::new("hi"), WordState::new("yo")];
        words[0].typed = "hi".chars().collect();
        words[1].typed = "yo".chars().collect();
        assert_eq!(count_chars_in(&words, 2), (4, 0));
    }

    #[test]
    fn count_chars_with_typo() {
        let mut words = vec![WordState::new("hello")];
        words[0].typed = "hellx".chars().collect();
        // h e l l correct, x incorrect
        assert_eq!(count_chars_in(&words, 1), (4, 1));
    }

    #[test]
    fn count_chars_with_extra_typed_chars() {
        let mut words = vec![WordState::new("hi")];
        words[0].typed = "hiXX".chars().collect();
        // 'h' and 'i' correct; 'X' and 'X' are extra → incorrect
        assert_eq!(count_chars_in(&words, 1), (2, 2));
    }

    #[test]
    fn count_chars_stops_at_current_word_boundary() {
        let mut words = vec![WordState::new("hi"), WordState::new("yo")];
        words[0].typed = "hi".chars().collect();
        words[1].typed = "yo".chars().collect();
        // Only count first word
        assert_eq!(count_chars_in(&words, 1), (2, 0));
    }

    #[test]
    fn count_chars_empty_words() {
        let words: Vec<WordState> = vec![];
        assert_eq!(count_chars_in(&words, 0), (0, 0));
    }

    // ---- calculate_stats ----

    #[test]
    fn calculate_stats_perfect_run() {
        // 60 correct chars in 60 s → 60/5 = 12 "words" / 1 min = 12 WPM
        let (wpm, raw_wpm, accuracy) = calculate_stats(60, 0, 60.0);
        assert!((wpm - 12.0).abs() < 0.01);
        assert!((raw_wpm - 12.0).abs() < 0.01);
        assert!((accuracy - 100.0).abs() < 0.01);
    }

    #[test]
    fn calculate_stats_with_errors() {
        // 80 correct + 20 incorrect in 60 s
        let (wpm, raw_wpm, accuracy) = calculate_stats(80, 20, 60.0);
        assert!((wpm - 16.0).abs() < 0.01);     // 80/5/1
        assert!((raw_wpm - 20.0).abs() < 0.01); // 100/5/1
        assert!((accuracy - 80.0).abs() < 0.01);
    }

    #[test]
    fn calculate_stats_no_chars_gives_100_accuracy() {
        let (_, _, accuracy) = calculate_stats(0, 0, 10.0);
        assert_eq!(accuracy, 100.0);
    }

    #[test]
    fn calculate_stats_faster_typing_gives_higher_wpm() {
        let (wpm30, _, _) = calculate_stats(60, 0, 30.0);
        let (wpm60, _, _) = calculate_stats(60, 0, 60.0);
        assert!(wpm30 > wpm60);
    }
}
