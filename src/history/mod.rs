use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::TestResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub wpm: f64,
    pub raw_wpm: f64,
    pub accuracy: f64,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub duration_secs: f64,
    pub mode: String,
    pub timestamp: u64,
}

impl HistoryEntry {
    pub fn from_result(result: &TestResult) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            wpm: result.wpm,
            raw_wpm: result.raw_wpm,
            accuracy: result.accuracy,
            correct_chars: result.correct_chars,
            incorrect_chars: result.incorrect_chars,
            duration_secs: result.duration.as_secs_f64(),
            mode: result.mode.clone(),
            timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct History {
    pub entries: Vec<HistoryEntry>,
}

impl History {
    fn data_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "monkeyterm")
            .map(|dirs| dirs.data_dir().join("history.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::data_path() {
            if path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(history) = serde_json::from_str::<History>(&contents) {
                        return history;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::data_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let contents = serde_json::to_string_pretty(self)?;
            std::fs::write(path, contents)?;
        }
        Ok(())
    }

    pub fn add(&mut self, result: &TestResult) {
        self.entries.push(HistoryEntry::from_result(result));
        if self.entries.len() > 1000 {
            let drain = self.entries.len() - 1000;
            self.entries.drain(0..drain);
        }
    }

    /// Personal best WPM for an exact mode string (e.g. "words 50", "time 60s").
    pub fn personal_best(&self, mode: &str) -> Option<&HistoryEntry> {
        self.entries
            .iter()
            .filter(|e| e.mode == mode)
            .max_by(|a, b| a.wpm.partial_cmp(&b.wpm).unwrap_or(std::cmp::Ordering::Equal))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entry(wpm: f64, mode: &str) -> HistoryEntry {
        HistoryEntry {
            wpm,
            raw_wpm: wpm,
            accuracy: 100.0,
            correct_chars: 50,
            incorrect_chars: 0,
            duration_secs: 30.0,
            mode: mode.to_string(),
            timestamp: 0,
        }
    }

    #[test]
    fn personal_best_empty_history() {
        let h = History::default();
        assert!(h.personal_best("words 50").is_none());
    }

    #[test]
    fn personal_best_returns_highest_wpm() {
        let mut h = History::default();
        h.entries.push(make_entry(60.0, "words 50"));
        h.entries.push(make_entry(80.0, "words 50"));
        h.entries.push(make_entry(70.0, "words 50"));
        let pb = h.personal_best("words 50").unwrap();
        assert_eq!(pb.wpm, 80.0);
    }

    #[test]
    fn personal_best_filters_by_mode() {
        let mut h = History::default();
        h.entries.push(make_entry(100.0, "time 60s"));
        h.entries.push(make_entry(50.0, "words 50"));
        // PB for words 50 should not be influenced by the time 60s entry
        let pb = h.personal_best("words 50").unwrap();
        assert_eq!(pb.wpm, 50.0);
        assert!(h.personal_best("time 60s").is_some());
    }

    #[test]
    fn personal_best_unknown_mode_returns_none() {
        let mut h = History::default();
        h.entries.push(make_entry(80.0, "words 50"));
        assert!(h.personal_best("words 25").is_none());
    }

    #[test]
    fn add_caps_history_at_1000_entries() {
        let mut h = History::default();
        // Pre-fill to 999 entries
        for i in 0..999 {
            h.entries.push(make_entry(i as f64, "words 50"));
        }
        assert_eq!(h.entries.len(), 999);

        // Adding one more brings us to 1000 — no drain yet
        h.entries.push(make_entry(999.0, "words 50"));
        assert_eq!(h.entries.len(), 1000);

        // Now add via add() which triggers the drain
        use std::time::Duration;
        use crate::app::TestResult;
        let result = TestResult {
            wpm: 85.0,
            raw_wpm: 90.0,
            accuracy: 98.0,
            correct_chars: 250,
            incorrect_chars: 5,
            duration: Duration::from_secs(30),
            mode: "words 50".to_string(),
        };
        h.add(&result);
        assert_eq!(h.entries.len(), 1000);
    }

    #[test]
    fn add_stores_correct_wpm() {
        use std::time::Duration;
        use crate::app::TestResult;
        let mut h = History::default();
        let result = TestResult {
            wpm: 75.5,
            raw_wpm: 80.0,
            accuracy: 95.0,
            correct_chars: 200,
            incorrect_chars: 10,
            duration: Duration::from_secs(60),
            mode: "words 100".to_string(),
        };
        h.add(&result);
        assert_eq!(h.entries.len(), 1);
        assert_eq!(h.entries[0].wpm, 75.5);
        assert_eq!(h.entries[0].mode, "words 100");
    }
}
