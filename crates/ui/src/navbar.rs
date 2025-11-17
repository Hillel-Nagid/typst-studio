use crate::{ components::{ Button, ButtonVariant }, theme::Theme };
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct NavBar {
    theme: Arc<RwLock<Theme>>,
    file_button: Entity<Button>,
    edit_button: Entity<Button>,
    view_button: Entity<Button>,
    help_button: Entity<Button>,
}

impl NavBar {
    pub fn new(theme: Arc<RwLock<Theme>>, cx: &mut Context<Self>) -> Self {
        let file_button = cx.new(|_cx| Button::new("File", ButtonVariant::Primary, theme.clone()));
        let edit_button = cx.new(|_cx| Button::new("Edit", ButtonVariant::Primary, theme.clone()));
        let view_button = cx.new(|_cx| Button::new("View", ButtonVariant::Primary, theme.clone()));
        let help_button = cx.new(|_cx| Button::new("Help", ButtonVariant::Primary, theme.clone()));
        Self { theme, file_button, edit_button, view_button, help_button }
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
                            .child(self.file_button.clone())
                            .child(self.edit_button.clone())
                            .child(self.view_button.clone())
                            .child(self.help_button.clone())
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
