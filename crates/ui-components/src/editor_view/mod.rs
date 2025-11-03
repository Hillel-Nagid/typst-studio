//! Editor view components
//!
//! Phase 3.1: Editor View Component Hierarchy

use gpui::*;
use editor_core::BufferId;

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
        }
    }

    pub fn set_buffer(&mut self, buffer_id: BufferId) {
        self.buffer_id = Some(buffer_id);
    }

    pub fn buffer_id(&self) -> Option<BufferId> {
        self.buffer_id
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
                            .child("Line 1, Col 1 | Typst | UTF-8 | LF | RTL/LTR: Enabled")
                            .text_color(rgb(0xffffff))
                            .text_size(px(12.0))
                    )
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_view_creation() {
        let view = EditorView::new();
        assert!(view.buffer_id.is_none());
    }
}
