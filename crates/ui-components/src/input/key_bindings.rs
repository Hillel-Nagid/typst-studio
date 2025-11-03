//! Key binding system for customizable shortcuts
//!
//! Phase 3.4: Input Handling

use serde::{ Deserialize, Serialize };
use std::collections::HashMap;

/// Key binding manager
pub struct KeyBindings {
    bindings: HashMap<KeyBinding, Action>,
}

impl KeyBindings {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    /// Load default key bindings for the current platform
    pub fn load_defaults() -> Self {
        let mut kb = Self::new();

        // Text input bindings (handled via text input event, not key events)

        // Cursor movement
        kb.register(KeyBinding::new("ArrowLeft", Modifiers::none()), Action::MoveLeft);
        kb.register(KeyBinding::new("ArrowRight", Modifiers::none()), Action::MoveRight);
        kb.register(KeyBinding::new("ArrowUp", Modifiers::none()), Action::MoveUp);
        kb.register(KeyBinding::new("ArrowDown", Modifiers::none()), Action::MoveDown);
        kb.register(KeyBinding::new("ArrowLeft", Modifiers::ctrl()), Action::MoveWordLeft);
        kb.register(KeyBinding::new("ArrowRight", Modifiers::ctrl()), Action::MoveWordRight);
        kb.register(KeyBinding::new("Home", Modifiers::none()), Action::MoveLineStart);
        kb.register(KeyBinding::new("End", Modifiers::none()), Action::MoveLineEnd);
        kb.register(KeyBinding::new("PageUp", Modifiers::none()), Action::MovePageUp);
        kb.register(KeyBinding::new("PageDown", Modifiers::none()), Action::MovePageDown);

        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("ArrowUp", Modifiers::cmd()), Action::MoveDocumentStart);
            kb.register(KeyBinding::new("ArrowDown", Modifiers::cmd()), Action::MoveDocumentEnd);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("Home", Modifiers::ctrl()), Action::MoveDocumentStart);
            kb.register(KeyBinding::new("End", Modifiers::ctrl()), Action::MoveDocumentEnd);
        }

        // Selection
        kb.register(KeyBinding::new("ArrowLeft", Modifiers::shift()), Action::SelectLeft);
        kb.register(KeyBinding::new("ArrowRight", Modifiers::shift()), Action::SelectRight);
        kb.register(KeyBinding::new("ArrowUp", Modifiers::shift()), Action::SelectUp);
        kb.register(KeyBinding::new("ArrowDown", Modifiers::shift()), Action::SelectDown);

        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("a", Modifiers::cmd()), Action::SelectAll);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("a", Modifiers::ctrl()), Action::SelectAll);
        }

        // Editing
        kb.register(KeyBinding::new("Delete", Modifiers::none()), Action::Delete);
        kb.register(KeyBinding::new("Backspace", Modifiers::none()), Action::Backspace);
        kb.register(KeyBinding::new("Enter", Modifiers::none()), Action::Newline);
        kb.register(KeyBinding::new("Tab", Modifiers::none()), Action::Indent);
        kb.register(KeyBinding::new("Tab", Modifiers::shift()), Action::Outdent);

        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("w", Modifiers::alt()), Action::DeleteWord);
            kb.register(KeyBinding::new("k", Modifiers::cmd()), Action::DeleteLine);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("Delete", Modifiers::ctrl()), Action::DeleteWord);
            kb.register(KeyBinding::new("k", Modifiers::ctrl()), Action::DeleteLine);
        }

        // Clipboard
        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("c", Modifiers::cmd()), Action::Copy);
            kb.register(KeyBinding::new("x", Modifiers::cmd()), Action::Cut);
            kb.register(KeyBinding::new("v", Modifiers::cmd()), Action::Paste);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("c", Modifiers::ctrl()), Action::Copy);
            kb.register(KeyBinding::new("x", Modifiers::ctrl()), Action::Cut);
            kb.register(KeyBinding::new("v", Modifiers::ctrl()), Action::Paste);
        }

        // Undo/Redo
        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("z", Modifiers::cmd()), Action::Undo);
            kb.register(KeyBinding::new("z", Modifiers::cmd().with_shift()), Action::Redo);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("z", Modifiers::ctrl()), Action::Undo);
            kb.register(KeyBinding::new("y", Modifiers::ctrl()), Action::Redo);
            kb.register(KeyBinding::new("z", Modifiers::ctrl().with_shift()), Action::Redo);
        }

        // File operations
        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("s", Modifiers::cmd()), Action::Save);
            kb.register(KeyBinding::new("o", Modifiers::cmd()), Action::Open);
            kb.register(KeyBinding::new("s", Modifiers::cmd().with_shift()), Action::SaveAs);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("s", Modifiers::ctrl()), Action::Save);
            kb.register(KeyBinding::new("o", Modifiers::ctrl()), Action::Open);
            kb.register(KeyBinding::new("s", Modifiers::ctrl().with_shift()), Action::SaveAs);
        }

        // Search
        #[cfg(target_os = "macos")]
        {
            kb.register(KeyBinding::new("f", Modifiers::cmd()), Action::Find);
            kb.register(KeyBinding::new("g", Modifiers::cmd()), Action::FindNext);
            kb.register(KeyBinding::new("g", Modifiers::cmd().with_shift()), Action::FindPrevious);
            kb.register(KeyBinding::new("h", Modifiers::cmd()), Action::Replace);
        }

        #[cfg(not(target_os = "macos"))]
        {
            kb.register(KeyBinding::new("f", Modifiers::ctrl()), Action::Find);
            kb.register(KeyBinding::new("g", Modifiers::ctrl()), Action::FindNext);
            kb.register(KeyBinding::new("g", Modifiers::ctrl().with_shift()), Action::FindPrevious);
            kb.register(KeyBinding::new("h", Modifiers::ctrl()), Action::Replace);
        }

        kb
    }

    /// Register a key binding
    pub fn register(&mut self, binding: KeyBinding, action: Action) {
        self.bindings.insert(binding, action);
    }

    /// Find action for a key event
    pub fn find_action(&self, key: &str, modifiers: Modifiers) -> Option<Action> {
        let binding = KeyBinding::new(key, modifiers);
        self.bindings.get(&binding).cloned()
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::load_defaults()
    }
}

/// A key binding (key combination)
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct KeyBinding {
    pub key: String,
    pub modifiers: Modifiers,
}

impl KeyBinding {
    pub fn new(key: &str, modifiers: Modifiers) -> Self {
        Self {
            key: key.to_string(),
            modifiers,
        }
    }
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,
}

impl Modifiers {
    pub fn none() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn ctrl() -> Self {
        Self {
            ctrl: true,
            alt: false,
            shift: false,
            meta: false,
        }
    }

    pub fn alt() -> Self {
        Self {
            ctrl: false,
            alt: true,
            shift: false,
            meta: false,
        }
    }

    pub fn shift() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: true,
            meta: false,
        }
    }

    pub fn cmd() -> Self {
        // On macOS, cmd is meta. On other platforms, ctrl is used.
        #[cfg(target_os = "macos")]
        {
            Self {
                ctrl: false,
                alt: false,
                shift: false,
                meta: true,
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                ctrl: true,
                alt: false,
                shift: false,
                meta: false,
            }
        }
    }

    pub fn with_shift(mut self) -> Self {
        self.shift = true;
        self
    }

    pub fn with_alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn with_ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    pub fn with_meta(mut self) -> Self {
        self.meta = true;
        self
    }
}

/// Editor actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    // Cursor movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordLeft,
    MoveWordRight,
    MoveLineStart,
    MoveLineEnd,
    MovePageUp,
    MovePageDown,
    MoveDocumentStart,
    MoveDocumentEnd,

    // Selection
    SelectLeft,
    SelectRight,
    SelectUp,
    SelectDown,
    SelectAll,

    // Editing
    Insert(String),
    Delete,
    Backspace,
    DeleteWord,
    DeleteLine,
    Newline,
    Indent,
    Outdent,

    // Clipboard
    Copy,
    Cut,
    Paste,

    // Undo/Redo
    Undo,
    Redo,

    // File operations
    Save,
    SaveAs,
    Open,
    Close,

    // Search
    Find,
    FindNext,
    FindPrevious,
    Replace,

    // Multi-cursor
    AddCursor,
    SelectNextOccurrence,

    // Custom action
    Custom(String),
}
