use crate::theme::Theme;
use editor_core::ApplicationState;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct EditorPanel {
    theme: Arc<RwLock<Theme>>,
    state: Arc<RwLock<ApplicationState>>,
}

impl EditorPanel {
    pub fn new(
        theme: Arc<RwLock<Theme>>,
        state: Arc<RwLock<ApplicationState>>,
        _cx: &mut Context<Self>
    ) -> Self {
        Self { theme, state }
    }
}

impl Render for EditorPanel {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.editor);
        let fg_color = theme.parse_color(&theme.foreground.editor);
        let gutter_bg = theme.parse_color(&theme.background.gutter);
        let gutter_fg = theme.parse_color(&theme.foreground.gutter);

        // Get active document content
        let content = if let Some(workspace) = self.state.read().get_active_workspace() {
            let workspace = workspace.read();
            if let Some(editor) = workspace.get_active_editor() {
                let editor = editor.read();
                if editor.content.is_empty() {
                    "// Welcome to Typst Studio\n// Start typing...".to_string()
                } else {
                    editor.content.clone()
                }
            } else {
                "// No document open".to_string()
            }
        } else {
            "// No workspace".to_string()
        };

        div()
            .flex_1()
            .flex()
            .flex_row()
            .bg(bg_color)
            .text_color(fg_color)
            // Line numbers gutter
            .child(
                div()
                    .w_12()
                    .h_full()
                    .bg(gutter_bg)
                    .text_color(gutter_fg)
                    .flex()
                    .flex_col()
                    .p_2()
                    .text_xs()
                    .children((1..=20).map(|i| div().child(format!("{}", i))))
            )
            // Editor content
            .child(
                div()
                    .flex_1()
                    .p_2()
                    .font_family("monospace")
                    .text_sm()
                    //TODO: add scroll on overflow
                    .child(div().whitespace_normal().child(content))
            )
    }
}
