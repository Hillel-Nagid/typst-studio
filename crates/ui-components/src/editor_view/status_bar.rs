//! Status bar component at the bottom of the editor
//!
//! Phase 3.1: Editor View Component Hierarchy

use editor_core::selection::Position;

/// Status bar at the bottom of the editor
pub struct StatusBar {
    /// Current cursor position
    pub cursor_position: Position,
    /// File name
    pub file_name: String,
    /// File encoding (default UTF-8)
    pub encoding: String,
    /// Line ending type
    pub line_ending: LineEndingDisplay,
    /// Language mode
    pub language: String,
    /// Diagnostic message
    pub diagnostic: Option<String>,
    /// Dirty flag (unsaved changes)
    pub dirty: bool,
}

/// Line ending display variant
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEndingDisplay {
    /// Unix line ending (LF)
    Lf,
    /// Windows line ending (CRLF)
    Crlf,
    /// Classic Mac line ending (CR)
    Cr,
}

impl LineEndingDisplay {
    pub fn as_str(&self) -> &str {
        match self {
            LineEndingDisplay::Lf => "LF",
            LineEndingDisplay::Crlf => "CRLF",
            LineEndingDisplay::Cr => "CR",
        }
    }
}

impl Default for LineEndingDisplay {
    fn default() -> Self {
        #[cfg(windows)]
        return LineEndingDisplay::Crlf;
        #[cfg(not(windows))]
        return LineEndingDisplay::Lf;
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            cursor_position: Position::zero(),
            file_name: "Untitled".to_string(),
            encoding: "UTF-8".to_string(),
            line_ending: LineEndingDisplay::default(),
            language: "Typst".to_string(),
            diagnostic: None,
            dirty: false,
        }
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = position;
    }

    /// Set file name
    pub fn set_file_name(&mut self, name: String) {
        self.file_name = name;
    }

    /// Set encoding
    pub fn set_encoding(&mut self, encoding: String) {
        self.encoding = encoding;
    }

    /// Set line ending type
    pub fn set_line_ending(&mut self, ending: LineEndingDisplay) {
        self.line_ending = ending;
    }

    /// Set language mode
    pub fn set_language(&mut self, language: String) {
        self.language = language;
    }

    /// Set diagnostic message
    pub fn set_diagnostic(&mut self, message: Option<String>) {
        self.diagnostic = message;
    }

    /// Set dirty flag
    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty;
    }

    /// Get status bar text
    pub fn get_status_text(&self) -> String {
        let mut parts = vec![];

        // Cursor position
        let pos_display = format!(
            "Line {}, Column {}",
            self.cursor_position.line + 1,
            self.cursor_position.column + 1
        );
        parts.push(pos_display);

        // Encoding
        parts.push(self.encoding.clone());

        parts.join(" | ")
    }

    /// Get diagnostic text if any
    pub fn get_diagnostic_text(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }

    /// Get error status indicator for display (checkmark or error count)
    pub fn get_error_status(&self) -> String {
        if let Some(diag) = &self.diagnostic {
            format!("✕ {}", diag)
        } else {
            "No errors ✓".to_string()
        }
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_bar_creation() {
        let sb = StatusBar::new();
        assert_eq!(sb.file_name, "Untitled");
        assert_eq!(sb.language, "Typst");
        assert!(!sb.dirty);
    }

    #[test]
    fn test_status_bar_text() {
        let mut sb = StatusBar::new();
        sb.set_cursor_position(Position::new(5, 10));
        sb.set_file_name("main.typ".to_string());

        let text = sb.get_status_text();
        assert!(text.contains("Line 6, Column 11"));
        assert!(text.contains("main.typ"));
    }

    #[test]
    fn test_status_bar_dirty() {
        let mut sb = StatusBar::new();
        sb.set_file_name("test.typ".to_string());
        sb.set_dirty(true);

        let text = sb.get_status_text();
        assert!(text.contains("●"));
    }

    #[test]
    fn test_line_ending_display() {
        assert_eq!(LineEndingDisplay::Lf.as_str(), "LF");
        assert_eq!(LineEndingDisplay::Crlf.as_str(), "CRLF");
        assert_eq!(LineEndingDisplay::Cr.as_str(), "CR");
    }
}
