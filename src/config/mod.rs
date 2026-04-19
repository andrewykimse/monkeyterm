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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::words::WordList;

    #[test]
    fn test_mode_roundtrip_for_all_named_modes() {
        let modes = [
            TestMode::Words(25),
            TestMode::Words(50),
            TestMode::Words(100),
            TestMode::Time(30),
            TestMode::Time(60),
            TestMode::Time(120),
            TestMode::Quote,
        ];
        for mode in &modes {
            let s = Config::test_mode_from(mode);
            let recovered = Config { default_mode: s.clone(), ..Config::default() }.test_mode();
            assert_eq!(&recovered, mode, "roundtrip failed for mode string {s:?}");
        }
    }

    #[test]
    fn test_mode_unknown_string_falls_back_to_words50() {
        let cfg = Config { default_mode: "garbage".to_string(), ..Config::default() };
        assert_eq!(cfg.test_mode(), TestMode::Words(50));
    }

    #[test]
    fn word_list_roundtrip() {
        for list in [WordList::Common200, WordList::Programming] {
            let s = Config::word_list_from(&list);
            let recovered = Config { word_list: s.clone(), ..Config::default() }.word_list();
            assert_eq!(recovered, list, "roundtrip failed for word list string {s:?}");
        }
    }

    #[test]
    fn word_list_unknown_string_falls_back_to_english() {
        let cfg = Config { word_list: "gibberish".to_string(), ..Config::default() };
        assert_eq!(cfg.word_list(), WordList::Common200);
    }

    #[test]
    fn default_config_is_valid() {
        let cfg = Config::default();
        // Should not panic and should produce usable values
        let _ = cfg.test_mode();
        let _ = cfg.word_list();
        assert!(!cfg.theme.is_empty());
    }
}
