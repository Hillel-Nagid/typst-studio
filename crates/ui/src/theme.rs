use gpui::Hsla;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub background: ThemeColors,
    pub foreground: ThemeColors,
    pub semantic: SemanticColors,
    pub ui: UiColors,
    pub syntax: SyntaxColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub editor: String,
    pub sidebar: String,
    pub preview: String,
    pub titlebar: String,
    pub panel: String,
    pub gutter: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticColors {
    pub error: String,
    pub warning: String,
    pub info: String,
    pub success: String,
    pub hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiColors {
    pub selection_background: String,
    pub selection_foreground: String,
    pub cursor: String,
    pub line_highlight: String,
    pub matching_bracket: String,
    pub button_background: String,
    pub button_hover: String,
    pub button_active: String,
    pub input_background: String,
    pub input_border: String,
    pub border: String,
    pub divider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxColors {
    pub keyword: String,
    pub function: String,
    pub variable: String,
    pub string: String,
    pub number: String,
    pub comment: String,
    pub type_name: String,
    pub operator: String,
    pub punctuation: String,
    pub heading: String,
    pub emphasis: String,
    pub strong: String,
    pub link: String,
    pub code: String,
}

impl Theme {
    pub fn dark() -> Self {
        Self {
            name: "Dark".to_string(),
            background: ThemeColors {
                editor: "#1e1e1e".to_string(),
                sidebar: "#252526".to_string(),
                preview: "#2d2d2d".to_string(),
                titlebar: "#323233".to_string(),
                panel: "#252526".to_string(),
                gutter: "#1e1e1e".to_string(),
            },
            foreground: ThemeColors {
                editor: "#d4d4d4".to_string(),
                sidebar: "#cccccc".to_string(),
                preview: "#ffffff".to_string(),
                titlebar: "#cccccc".to_string(),
                panel: "#cccccc".to_string(),
                gutter: "#858585".to_string(),
            },
            semantic: SemanticColors {
                error: "#f48771".to_string(),
                warning: "#cca700".to_string(),
                info: "#75beff".to_string(),
                success: "#89d185".to_string(),
                hint: "#eeeeeeb3".to_string(),
            },
            ui: UiColors {
                selection_background: "#264f78".to_string(),
                selection_foreground: "#ffffff".to_string(),
                cursor: "#aeafad".to_string(),
                line_highlight: "#ffffff0f".to_string(),
                matching_bracket: "#0064001a".to_string(),
                button_background: "#0e639c".to_string(),
                button_hover: "#1177bb".to_string(),
                button_active: "#0e639c".to_string(),
                input_background: "#3c3c3c".to_string(),
                input_border: "#3c3c3c".to_string(),
                border: "#454545".to_string(),
                divider: "#454545".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#569cd6".to_string(),
                function: "#dcdcaa".to_string(),
                variable: "#9cdcfe".to_string(),
                string: "#ce9178".to_string(),
                number: "#b5cea8".to_string(),
                comment: "#6a9955".to_string(),
                type_name: "#4ec9b0".to_string(),
                operator: "#d4d4d4".to_string(),
                punctuation: "#d4d4d4".to_string(),
                heading: "#569cd6".to_string(),
                emphasis: "#d7ba7d".to_string(),
                strong: "#d7ba7d".to_string(),
                link: "#3794ff".to_string(),
                code: "#ce9178".to_string(),
            },
        }
    }

    pub fn light() -> Self {
        Self {
            name: "Light".to_string(),
            background: ThemeColors {
                editor: "#ffffff".to_string(),
                sidebar: "#f3f3f3".to_string(),
                preview: "#f8f8f8".to_string(),
                titlebar: "#ececec".to_string(),
                panel: "#f3f3f3".to_string(),
                gutter: "#ffffff".to_string(),
            },
            foreground: ThemeColors {
                editor: "#000000".to_string(),
                sidebar: "#1e1e1e".to_string(),
                preview: "#000000".to_string(),
                titlebar: "#333333".to_string(),
                panel: "#1e1e1e".to_string(),
                gutter: "#237893".to_string(),
            },
            semantic: SemanticColors {
                error: "#e51400".to_string(),
                warning: "#bf8803".to_string(),
                info: "#1a85ff".to_string(),
                success: "#14ce14".to_string(),
                hint: "#6c6c6c".to_string(),
            },
            ui: UiColors {
                selection_background: "#add6ff".to_string(),
                selection_foreground: "#000000".to_string(),
                cursor: "#000000".to_string(),
                line_highlight: "#0000000c".to_string(),
                matching_bracket: "#00ff002a".to_string(),
                button_background: "#007acc".to_string(),
                button_hover: "#0098ff".to_string(),
                button_active: "#005a9e".to_string(),
                input_background: "#ffffff".to_string(),
                input_border: "#cecece".to_string(),
                border: "#e7e7e7".to_string(),
                divider: "#e7e7e7".to_string(),
            },
            syntax: SyntaxColors {
                keyword: "#0000ff".to_string(),
                function: "#795e26".to_string(),
                variable: "#001080".to_string(),
                string: "#a31515".to_string(),
                number: "#098658".to_string(),
                comment: "#008000".to_string(),
                type_name: "#267f99".to_string(),
                operator: "#000000".to_string(),
                punctuation: "#000000".to_string(),
                heading: "#0000ff".to_string(),
                emphasis: "#811f3f".to_string(),
                strong: "#811f3f".to_string(),
                link: "#0000ff".to_string(),
                code: "#a31515".to_string(),
            },
        }
    }

    pub fn parse_color(&self, color_str: &str) -> Hsla {
        // Parse hex color string to GPUI color
        if let Some(stripped) = color_str.strip_prefix('#') {
            let r = u8::from_str_radix(&stripped[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&stripped[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&stripped[4..6], 16).unwrap_or(0) as f32 / 255.0;
            let a = if stripped.len() > 6 {
                u8::from_str_radix(&stripped[6..8], 16).unwrap_or(255) as f32 / 255.0
            } else {
                1.0
            };
            Hsla::from_rgb(r, g, b, a)
        } else {
            Hsla::default()
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

