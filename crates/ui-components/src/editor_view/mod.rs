//! Editor view components
//!
//! Phase 3.1: Editor View Component Hierarchy

use gpui::*;
use editor_core::{ BufferId, Position, SelectionSet, Selection };

pub mod gutter;
pub mod text_content;
pub mod line_renderer;
pub mod cursor_renderer;
pub mod scrollbar;
pub mod overlays;
pub mod status_bar;
pub mod menu_bar;
pub mod top_nav;

pub use gutter::Gutter;
pub use text_content::TextContent;
pub use line_renderer::LineRenderer;
pub use cursor_renderer::{
    CursorRenderer,
    CursorShape,
    CursorStyle,
    PrimaryCursor,
    SecondaryCursors,
};
pub use scrollbar::ScrollBar;
pub use overlays::Overlays;
pub use status_bar::StatusBar;
pub use menu_bar::{ MenuBar, Menu, MenuItem };
pub use top_nav::TopNav;

/// Editor view component - the main editor interface
pub struct EditorView {
    /// Current buffer ID
    buffer_id: Option<BufferId>,
    /// Gutter component
    pub gutter: Gutter,
    /// Text content area
    pub text_content: TextContent,
    /// Cursor renderer
    pub cursor_renderer: CursorRenderer,
    /// Scrollbar
    pub scrollbar: ScrollBar,
    /// Status bar
    pub status_bar: StatusBar,
    /// View state
    pub scroll_offset: f32,
    /// Current cursor position
    pub cursor_position: Position,
    /// Current selection set
    pub selection: SelectionSet,
}

impl EditorView {
    pub fn new() -> Self {
        Self {
            buffer_id: None,
            gutter: Gutter::new(),
            text_content: TextContent::new(),
            cursor_renderer: CursorRenderer::new(),
            scrollbar: ScrollBar::new(),
            status_bar: StatusBar::new(),
            scroll_offset: 0.0,
            cursor_position: Position::zero(),
            selection: SelectionSet::new(Selection::collapsed(Position::zero())),
        }
    }

    pub fn set_buffer(&mut self, buffer_id: BufferId) {
        self.buffer_id = Some(buffer_id);
    }

    pub fn buffer_id(&self) -> Option<BufferId> {
        self.buffer_id
    }

    /// Set cursor position
    pub fn set_cursor_position(&mut self, position: Position) {
        self.cursor_position = position;
        self.cursor_renderer.on_cursor_moved();
    }

    /// Move cursor by offset
    pub fn move_cursor(&mut self, line_delta: isize, column_delta: isize) {
        let new_line = ((self.cursor_position.line as isize) + line_delta).max(0) as usize;
        let new_column = ((self.cursor_position.column as isize) + column_delta).max(0) as usize;
        self.set_cursor_position(Position::new(new_line, new_column));
    }

    /// Update selection set
    pub fn set_selection(&mut self, selection: SelectionSet) {
        self.selection = selection;
    }

    /// Get current cursor position
    pub fn get_cursor_position(&self) -> Position {
        self.cursor_position
    }

    /// Get current selection
    pub fn get_selection(&self) -> &SelectionSet {
        &self.selection
    }

    /// Get mutable selection
    pub fn get_selection_mut(&mut self) -> &mut SelectionSet {
        &mut self.selection
    }

    /// Map mouse coordinates to buffer position
    /// gutter_width: width of line number gutter in pixels
    /// content_x, content_y: mouse coordinates relative to text area start
    /// line_height and char_width: from text_content metrics
    pub fn point_to_position(
        content_x: f32,
        content_y: f32,
        char_width: f32,
        line_height: f32
    ) -> Position {
        let line = (content_y / line_height).floor() as usize;
        let column = (content_x / char_width).floor() as usize;
        Position::new(line, column)
    }
}

impl Default for EditorView {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for EditorView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            // Main editor content area
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    // Gutter
                    .child(
                        div()
                            .w(px(self.gutter.calculate_width(100)))
                            .h_full()
                            .bg(rgb(0x252526))
                            .flex()
                            .flex_col()
                            .overflow_hidden()
                            .px(px(4.0))
                            .py(px(8.0))
                            .children(
                                (0..20).map(|line| {
                                    div()
                                        .h(px(self.text_content.line_height))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            div()
                                                .child(format!("{}", line + 1))
                                                .text_color(rgb(0x858585))
                                                .text_size(px(12.0))
                                        )
                                })
                            )
                    )
                    // Text content area with bidirectional text rendering
                    .child(
                        div()
                            .flex_1()
                            .flex_col()
                            .px(px(8.0))
                            .py(px(8.0))
                            .children(
                                (0..20).map(|_| {
                                    div()
                                        .h(px(self.text_content.line_height))
                                        .child(
                                            div()
                                                // NOTE: This is sample text demonstrating bidi support.
                                                // In production, this would render actual buffer content
                                                // through the shape_with_bidi pipeline.
                                                .child("// Mixed text: English אבג 123 عرب")
                                                .text_color(rgb(0x6a9955))
                                                .text_size(px(13.0))
                                        )
                                })
                            )
                    )
                    // Scrollbar
                    .child(
                        div()
                            .w(px(12.0))
                            .h_full()
                            .bg(rgb(0x1e1e1e))
                            .flex()
                            .justify_center()
                            .py(px(2.0))
                            .child(div().w(px(8.0)).h(px(60.0)).rounded(px(4.0)).bg(rgb(0x464647)))
                    )
            )
            // Status bar
            .child(
                div()
                    .w_full()
                    .h(px(24.0))
                    .bg(rgb(0x007acc))
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .child(
                        div()
                            .child(
                                format!(
                                    "Line {}, Col {} | Typst | UTF-8 | LF | RTL/LTR: Enabled",
                                    self.cursor_position.line + 1,
                                    self.cursor_position.column + 1
                                )
                            )
                            .text_color(rgb(0xffffff))
                            .text_size(px(12.0))
                    )
            )
    }
}
