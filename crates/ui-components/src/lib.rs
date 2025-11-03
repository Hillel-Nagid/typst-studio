//! UI components for Typst Studio
//!
//! This crate contains all the UI components for Typst Studio,
//! organized by functionality as defined in the project plan.

#![allow(dead_code)]
#![recursion_limit = "512"]

// Main component modules
pub mod editor_view;
pub mod preview_pane;
pub mod sidebar;
pub mod panels;
pub mod panels_layout;

// Phase 3 modules
pub mod rendering; // Phase 3.2: Text Rendering Pipeline
pub mod syntax; // Phase 3.3: Syntax Highlighting
pub mod input; // Phase 3.4: Input Handling
pub mod decorations; // Phase 3.5: Decorations and Annotations

// Re-export main components
pub use editor_view::EditorView;
pub use preview_pane::PreviewPane;
pub use sidebar::Sidebar;
pub use panels::Panel;

pub use decorations::{
    DecorationManager,
    InlineDecoration,
    InlineDecorationKind,
    GutterDecoration,
    GutterDecorationKind,
    HighlightRange,
    HighlightKind,
};
pub use input::{ InputHandler, KeyBindings };
pub use rendering::{ TextShaper, FontManager, LineLayout, Viewport };
pub use syntax::{ SyntaxHighlighter, Theme, ThemeManager };
