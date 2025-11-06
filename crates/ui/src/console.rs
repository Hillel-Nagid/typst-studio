use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ConsolePanel {
    theme: Arc<RwLock<Theme>>,
}

impl ConsolePanel {
    pub fn new(theme: Arc<RwLock<Theme>>, cx: &mut Context) -> Self {
        Self { theme }
    }
}

impl Render for ConsolePanel {
    fn render(&mut self, cx: &mut Context) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.panel);
        let fg_color = theme.parse_color(&theme.foreground.panel);

        div()
            .h_48()
            .w_full()
            .bg(bg_color)
            .text_color(fg_color)
            .flex()
            .flex_col()
            .border_t_1()
            .border_color(theme.parse_color(&theme.ui.border))
            // Console header
            .child(
                div()
                    .h_8()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .px_2()
                    .border_b_1()
                    .border_color(theme.parse_color(&theme.ui.border))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_4()
                            .text_sm()
                            .child(div().font_bold().child("Problems"))
                            .child(div().opacity(0.7).child("Output"))
                            .child(div().opacity(0.7).child("Terminal"))
                    )
                    .child(div().text_sm().child("✕"))
            )
            // Console content
            .child(
                div()
                    .flex_1()
                    .p_2()
                    .overflow_y_scroll()
                    .text_sm()
                    .child(div().opacity(0.6).child("No problems detected"))
            )
    }
}
