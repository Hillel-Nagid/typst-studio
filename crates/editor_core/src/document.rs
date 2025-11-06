use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocumentId(uuid::Uuid);

impl DocumentId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for DocumentId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Typst,
    Markdown,
    PlainText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Encoding {
    Utf8,
    Utf8Bom,
    Utf16Le,
    Utf16Be,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    Lf,    // Unix: \n
    CrLf,  // Windows: \r\n
    Cr,    // Old Mac: \r
}

impl LineEnding {
    #[cfg(target_os = "windows")]
    pub fn default_for_platform() -> Self {
        Self::CrLf
    }

    #[cfg(not(target_os = "windows"))]
    pub fn default_for_platform() -> Self {
        Self::Lf
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub id: DocumentId,
    pub path: Option<PathBuf>,
    pub language: Language,
    pub encoding: Encoding,
    pub line_ending: LineEnding,
    pub modified_time: Option<SystemTime>,
    pub is_dirty: bool,
    pub is_read_only: bool,
    pub version: u64,
}

impl Document {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self {
            id: DocumentId::new(),
            path,
            language: Language::Typst,
            encoding: Encoding::Utf8,
            line_ending: LineEnding::default_for_platform(),
            modified_time: None,
            is_dirty: false,
            is_read_only: false,
            version: 0,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.is_dirty = false;
    }

    pub fn increment_version(&mut self) {
        self.version += 1;
    }

    pub fn file_name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "Untitled".to_string())
    }
}

