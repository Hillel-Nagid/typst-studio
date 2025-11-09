use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct PreviewPane {
    theme: Arc<RwLock<Theme>>,
}

impl PreviewPane {
    pub fn new(theme: Arc<RwLock<Theme>>, cx: &mut Context<Self>) -> Self {
        Self { theme }
    }
}

impl Render for PreviewPane {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.preview);
        let fg_color = theme.parse_color(&theme.foreground.preview);

        div()
            .flex_1()
            .bg(bg_color)
            .text_color(fg_color)
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_4()
                    .child(div().text_2xl().child("Preview"))
                    .child(div().text_sm().opacity(0.7).child("PDF preview will appear here"))
            )
    }
}
