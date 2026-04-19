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
