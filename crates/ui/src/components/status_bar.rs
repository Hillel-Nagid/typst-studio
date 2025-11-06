use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct StatusBar {
    theme: Arc<RwLock<Theme>>,
    left_items: Vec<String>,
    right_items: Vec<String>,
}

impl StatusBar {
    pub fn new(theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            theme,
            left_items: vec!["Typst".to_string(), "Line 1, Col 1".to_string()],
            right_items: vec!["UTF-8".to_string(), "LF".to_string()],
        }
    }

    pub fn set_position(&mut self, line: usize, col: usize) {
        self.left_items[1] = format!("Line {}, Col {}", line + 1, col + 1);
    }
}

impl Render for StatusBar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.panel);
        let fg_color = theme.parse_color(&theme.foreground.panel);

        div()
            .h_6()
            .w_full()
            .bg(bg_color)
            .text_color(fg_color)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_2()
            .text_xs()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .children(self.left_items.iter().map(|item| div().child(item.clone()))),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .children(self.right_items.iter().map(|item| div().child(item.clone()))),
            )
    }
}

