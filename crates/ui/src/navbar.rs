use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct NavBar {
    theme: Arc<RwLock<Theme>>,
}

impl NavBar {
    pub fn new(theme: Arc<RwLock<Theme>>, _cx: &mut Context<Self>) -> Self {
        Self { theme }
    }
}

impl Render for NavBar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.titlebar);
        let fg_color = theme.parse_color(&theme.foreground.titlebar);

        div()
            .h_10()
            .w_full()
            .bg(bg_color)
            .text_color(fg_color)
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px_4()
            .border_b_1()
            .border_color(theme.parse_color(&theme.ui.border))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_4()
                    .items_center()
                    .child(div().font_weight(FontWeight::BOLD).text_lg().child("Typst Studio"))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap_2()
                            .text_sm()
                            .child(div().child("File"))
                            .child(div().child("Edit"))
                            .child(div().child("View"))
                            .child(div().child("Help"))
                    )
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_2()
                    .text_sm()
                    .child(div().child("🔍"))
                    .child(div().child("⚙️"))
            )
    }
}
