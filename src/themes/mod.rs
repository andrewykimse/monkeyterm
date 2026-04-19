use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub bg: ThemeColor,
    pub fg: ThemeColor,
    pub main: ThemeColor,   // accent / typed correctly
    pub sub: ThemeColor,    // untyped text / secondary
    pub error: ThemeColor,  // mistyped characters
    pub caret: ThemeColor,  // cursor
}

/// A color that can be serialized as a hex string (#rrggbb) and converted to ratatui's Color.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColor(pub String);

impl ThemeColor {
    pub fn to_color(&self) -> Color {
        let hex = self.0.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                return Color::Rgb(r, g, b);
            }
        }
        Color::Reset
    }
}

impl Theme {
    pub fn all() -> Vec<Theme> {
        vec![
            // MonkeyType default (serika dark)
            Theme {
                name: "serika_dark".into(),
                bg: ThemeColor("#323437".into()),
                fg: ThemeColor("#d1d0c5".into()),
                main: ThemeColor("#e2b714".into()),
                sub: ThemeColor("#646669".into()),
                error: ThemeColor("#ca4754".into()),
                caret: ThemeColor("#e2b714".into()),
            },
            // Dracula
            Theme {
                name: "dracula".into(),
                bg: ThemeColor("#282a36".into()),
                fg: ThemeColor("#f8f8f2".into()),
                main: ThemeColor("#bd93f9".into()),
                sub: ThemeColor("#6272a4".into()),
                error: ThemeColor("#ff5555".into()),
                caret: ThemeColor("#f8f8f2".into()),
            },
            // Nord
            Theme {
                name: "nord".into(),
                bg: ThemeColor("#2e3440".into()),
                fg: ThemeColor("#eceff4".into()),
                main: ThemeColor("#88c0d0".into()),
                sub: ThemeColor("#4c566a".into()),
                error: ThemeColor("#bf616a".into()),
                caret: ThemeColor("#88c0d0".into()),
            },
            // Catppuccin Mocha
            Theme {
                name: "catppuccin".into(),
                bg: ThemeColor("#1e1e2e".into()),
                fg: ThemeColor("#cdd6f4".into()),
                main: ThemeColor("#cba6f7".into()),
                sub: ThemeColor("#585b70".into()),
                error: ThemeColor("#f38ba8".into()),
                caret: ThemeColor("#f5c2e7".into()),
            },
            // Solarized Dark
            Theme {
                name: "solarized_dark".into(),
                bg: ThemeColor("#002b36".into()),
                fg: ThemeColor("#839496".into()),
                main: ThemeColor("#2aa198".into()),
                sub: ThemeColor("#073642".into()),
                error: ThemeColor("#dc322f".into()),
                caret: ThemeColor("#268bd2".into()),
            },
            // Gruvbox
            Theme {
                name: "gruvbox".into(),
                bg: ThemeColor("#282828".into()),
                fg: ThemeColor("#ebdbb2".into()),
                main: ThemeColor("#b8bb26".into()),
                sub: ThemeColor("#504945".into()),
                error: ThemeColor("#cc241d".into()),
                caret: ThemeColor("#fabd2f".into()),
            },
            // Monokai
            Theme {
                name: "monokai".into(),
                bg: ThemeColor("#272822".into()),
                fg: ThemeColor("#f8f8f2".into()),
                main: ThemeColor("#a6e22e".into()),
                sub: ThemeColor("#75715e".into()),
                error: ThemeColor("#f92672".into()),
                caret: ThemeColor("#e6db74".into()),
            },
            // One Dark
            Theme {
                name: "one_dark".into(),
                bg: ThemeColor("#282c34".into()),
                fg: ThemeColor("#abb2bf".into()),
                main: ThemeColor("#61afef".into()),
                sub: ThemeColor("#5c6370".into()),
                error: ThemeColor("#e06c75".into()),
                caret: ThemeColor("#528bff".into()),
            },
            // Tokyo Night
            Theme {
                name: "tokyo_night".into(),
                bg: ThemeColor("#1a1b26".into()),
                fg: ThemeColor("#a9b1d6".into()),
                main: ThemeColor("#7aa2f7".into()),
                sub: ThemeColor("#3b3d57".into()),
                error: ThemeColor("#f7768e".into()),
                caret: ThemeColor("#7dcfff".into()),
            },
            // Rose Pine
            Theme {
                name: "rose_pine".into(),
                bg: ThemeColor("#191724".into()),
                fg: ThemeColor("#e0def4".into()),
                main: ThemeColor("#c4a7e7".into()),
                sub: ThemeColor("#403d52".into()),
                error: ThemeColor("#eb6f92".into()),
                caret: ThemeColor("#31748f".into()),
            },
        ]
    }

    pub fn by_name(name: &str) -> Option<Theme> {
        Self::all().into_iter().find(|t| t.name == name)
    }

    pub fn default_theme() -> Theme {
        Self::all().into_iter().next().unwrap()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::Color;

    #[test]
    fn theme_color_parses_valid_hex_with_hash() {
        let c = ThemeColor("#ff8800".to_string());
        assert_eq!(c.to_color(), Color::Rgb(255, 136, 0));
    }

    #[test]
    fn theme_color_parses_valid_hex_without_hash() {
        let c = ThemeColor("00ff00".to_string());
        assert_eq!(c.to_color(), Color::Rgb(0, 255, 0));
    }

    #[test]
    fn theme_color_invalid_hex_returns_reset() {
        let c = ThemeColor("notacolor".to_string());
        assert_eq!(c.to_color(), Color::Reset);
    }

    #[test]
    fn theme_color_black_and_white() {
        assert_eq!(ThemeColor("#000000".to_string()).to_color(), Color::Rgb(0, 0, 0));
        assert_eq!(ThemeColor("#ffffff".to_string()).to_color(), Color::Rgb(255, 255, 255));
    }

    #[test]
    fn theme_all_is_nonempty() {
        assert!(!Theme::all().is_empty());
    }

    #[test]
    fn theme_all_names_are_unique() {
        let themes = Theme::all();
        let mut names: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        let original_len = names.len();
        names.dedup();
        // Sort first for dedup to catch all duplicates
        let mut sorted: Vec<&str> = themes.iter().map(|t| t.name.as_str()).collect();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), original_len, "duplicate theme names found");
    }

    #[test]
    fn theme_by_name_finds_known_themes() {
        assert!(Theme::by_name("serika_dark").is_some());
        assert!(Theme::by_name("dracula").is_some());
        assert!(Theme::by_name("nord").is_some());
    }

    #[test]
    fn theme_by_name_returns_none_for_unknown() {
        assert!(Theme::by_name("nonexistent_theme_xyz").is_none());
    }

    #[test]
    fn theme_by_name_returns_correct_theme() {
        let theme = Theme::by_name("dracula").unwrap();
        assert_eq!(theme.name, "dracula");
    }

    #[test]
    fn default_theme_is_serika_dark() {
        let theme = Theme::default_theme();
        assert_eq!(theme.name, "serika_dark");
    }
}
