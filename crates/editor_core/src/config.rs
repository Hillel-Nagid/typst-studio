use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub editor: EditorConfig,

    #[serde(default)]
    pub appearance: AppearanceConfig,

    #[serde(default)]
    pub lsp: LspConfig,

    #[serde(default)]
    pub compiler: CompilerConfig,

    #[serde(default)]
    pub bidi: BidiConfig,

    #[serde(default)]
    pub keybindings: HashMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            editor: EditorConfig::default(),
            appearance: AppearanceConfig::default(),
            lsp: LspConfig::default(),
            compiler: CompilerConfig::default(),
            bidi: BidiConfig::default(),
            keybindings: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorConfig {
    #[serde(default = "default_font_family")]
    pub font_family: String,

    #[serde(default = "default_font_size")]
    pub font_size: u32,

    #[serde(default = "default_line_height")]
    pub line_height: f32,

    #[serde(default = "default_tab_size")]
    pub tab_size: u32,

    #[serde(default = "default_true")]
    pub insert_spaces: bool,

    #[serde(default)]
    pub word_wrap: bool,

    #[serde(default = "default_true")]
    pub line_numbers: bool,

    #[serde(default = "default_true")]
    pub minimap: bool,

    #[serde(default = "default_cursor_style")]
    pub cursor_style: CursorStyle,

    #[serde(default = "default_true")]
    pub cursor_blink: bool,

    #[serde(default)]
    pub auto_save: bool,

    #[serde(default = "default_true")]
    pub auto_closing_brackets: bool,

    #[serde(default = "default_true")]
    pub auto_closing_quotes: bool,
}

impl Default for EditorConfig {
    fn default() -> Self {
        Self {
            font_family: default_font_family(),
            font_size: default_font_size(),
            line_height: default_line_height(),
            tab_size: default_tab_size(),
            insert_spaces: true,
            word_wrap: false,
            line_numbers: true,
            minimap: true,
            cursor_style: default_cursor_style(),
            cursor_blink: true,
            auto_save: false,
            auto_closing_brackets: true,
            auto_closing_quotes: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CursorStyle {
    Block,
    Line,
    Underline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,

    #[serde(default = "default_sidebar_position")]
    pub sidebar_position: SidebarPosition,
}

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
            ui_scale: default_ui_scale(),
            sidebar_position: default_sidebar_position(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    Left,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_lsp_server_path")]
    pub server_path: String,

    #[serde(default = "default_hover_delay")]
    pub hover_delay: u32,

    #[serde(default = "default_completion_triggers")]
    pub completion_triggers: Vec<String>,
}

impl Default for LspConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_path: default_lsp_server_path(),
            hover_delay: default_hover_delay(),
            completion_triggers: default_completion_triggers(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    #[serde(default = "default_true")]
    pub auto_compile_on_save: bool,

    #[serde(default = "default_true")]
    pub auto_compile_on_change: bool,

    #[serde(default = "default_compilation_delay")]
    pub compilation_delay: u32,

    #[serde(default)]
    pub show_compilation_output: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            auto_compile_on_save: true,
            auto_compile_on_change: true,
            compilation_delay: default_compilation_delay(),
            show_compilation_output: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_true")]
    pub rtl_line_alignment: bool,

    #[serde(default = "default_true")]
    pub math_mode_detection: bool,
}

impl Default for BidiConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rtl_line_alignment: true,
            math_mode_detection: true,
        }
    }
}

// Default value functions
fn default_font_family() -> String {
    "Fira Code".to_string()
}
fn default_font_size() -> u32 {
    14
}
fn default_line_height() -> f32 {
    1.5
}
fn default_tab_size() -> u32 {
    4
}
fn default_cursor_style() -> CursorStyle {
    CursorStyle::Line
}
fn default_theme() -> String {
    "dark".to_string()
}
fn default_ui_scale() -> f32 {
    1.0
}
fn default_sidebar_position() -> SidebarPosition {
    SidebarPosition::Left
}
fn default_lsp_server_path() -> String {
    "typst-lsp".to_string()
}
fn default_hover_delay() -> u32 {
    500
}
fn default_completion_triggers() -> Vec<String> {
    vec!["#".to_string(), ".".to_string(), ":".to_string()]
}
fn default_compilation_delay() -> u32 {
    500
}
fn default_true() -> bool {
    true
}

impl Config {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;

        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::from_str(&contents).map_err(Into::into)
        } else {
            serde_json::from_str(&contents).map_err(Into::into)
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let contents = if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            toml::to_string_pretty(self)?
        } else {
            serde_json::to_string_pretty(self)?
        };

        std::fs::write(path, contents)?;
        Ok(())
    }

    pub fn global_config_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("com", "typst", "typst-studio")
            .map(|dirs| dirs.config_dir().join("config.toml"))
    }

    pub fn load() -> Self {
        Self::global_config_path()
            .and_then(|path| Self::load_from_file(&path).ok())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },

    #[error("Missing required key: {key}")]
    MissingRequired { key: String },

    #[error("Parse error: {message}")]
    ParseError { message: String },
}

