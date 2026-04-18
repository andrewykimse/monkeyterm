use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::app::TestMode;
use crate::words::WordList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Name of the selected theme (e.g. "serika_dark", "dracula")
    pub theme: String,
    /// Word list identifier: "english" or "code"
    pub word_list: String,
    /// Default test mode: "words:25", "words:50", "words:100",
    ///                    "time:30", "time:60", "time:120", "quote"
    pub default_mode: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            theme: "serika_dark".into(),
            word_list: "english".into(),
            default_mode: "words:50".into(),
        }
    }
}

impl Config {
    /// Path to the config file, using the OS-appropriate config directory.
    pub fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "monkeyterm")
            .map(|dirs| dirs.config_dir().join("config.json"))
    }

    /// Load config from disk. Falls back to `Default` if the file is missing or invalid.
    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(contents) = std::fs::read_to_string(&path) {
                    if let Ok(cfg) = serde_json::from_str::<Config>(&contents) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }

    /// Persist the current config to disk.
    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let contents = serde_json::to_string_pretty(self)?;
            std::fs::write(path, contents)?;
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helpers to convert between config strings and app types
    // -----------------------------------------------------------------------

    pub fn word_list(&self) -> WordList {
        match self.word_list.as_str() {
            "code" => WordList::Programming,
            _ => WordList::Common200,
        }
    }

    pub fn word_list_from(list: &WordList) -> String {
        list.name().to_string()
    }

    pub fn test_mode(&self) -> TestMode {
        match self.default_mode.as_str() {
            "words:25" => TestMode::Words(25),
            "words:100" => TestMode::Words(100),
            "time:30" => TestMode::Time(30),
            "time:60" => TestMode::Time(60),
            "time:120" => TestMode::Time(120),
            "quote" => TestMode::Quote,
            _ => TestMode::Words(50),
        }
    }

    pub fn test_mode_from(mode: &TestMode) -> String {
        match mode {
            TestMode::Words(25) => "words:25".into(),
            TestMode::Words(50) => "words:50".into(),
            TestMode::Words(100) => "words:100".into(),
            TestMode::Time(30) => "time:30".into(),
            TestMode::Time(60) => "time:60".into(),
            TestMode::Time(120) => "time:120".into(),
            TestMode::Quote => "quote".into(),
            TestMode::Words(n) => format!("words:{n}"),
            TestMode::Time(s) => format!("time:{s}"),
            TestMode::Zen => "zen".into(),
        }
    }
}
