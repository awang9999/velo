// Configuration system
// Requirements 13, 15

use std::collections::HashMap;

/// VeloConfig is the fully-merged in-memory configuration object
/// Requirement 3.3
#[derive(Debug, Clone)]
pub struct VeloConfig {
    pub editor: EditorSettings,
    pub theme: ThemeConfig,
    pub keymaps: KeymapConfig,
    pub ui: UiConfig,
    pub plugins: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone)]
pub struct EditorSettings {
    pub tab_width: usize,
    pub line_numbers: bool,
    pub soft_wrap: bool,
    pub scroll_off: usize,
}

#[derive(Debug, Clone)]
pub struct ThemeConfig {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct KeymapConfig {
    pub global: HashMap<String, String>,
    pub by_major_mode: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct UiConfig {
    pub ui_type: String,
}

impl Default for VeloConfig {
    fn default() -> Self {
        Self {
            editor: EditorSettings::default(),
            theme: ThemeConfig::default(),
            keymaps: KeymapConfig::default(),
            ui: UiConfig::default(),
            plugins: HashMap::new(),
        }
    }
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_numbers: true,
            soft_wrap: false,
            scroll_off: 8,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "velo-dark".to_string(),
        }
    }
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self {
            global: HashMap::new(),
            by_major_mode: HashMap::new(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            ui_type: "velo-tui".to_string(),
        }
    }
}
