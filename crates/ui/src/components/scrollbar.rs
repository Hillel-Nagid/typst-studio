use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Scrollbar {
    theme: Arc<RwLock<Theme>>,
    position: f32,
    size: f32,
}

impl Scrollbar {
    pub fn new(theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            theme,
            position: 0.0,
            size: 0.2,
        }
    }

    pub fn set_position(&mut self, position: f32) {
        self.position = position.clamp(0.0, 1.0 - self.size);
    }

    pub fn set_size(&mut self, size: f32) {
        self.size = size.clamp(0.0, 1.0);
    }
}

impl Render for Scrollbar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.gutter);
        let thumb_color = theme.parse_color(&theme.ui.border);

        div()
            .w_3()
            .h_full()
            .bg(bg_color)
            .child(
                div()
                    .w_full()
                    .h(relative(self.size))
                    .bg(thumb_color)
                    .rounded_sm()
                    .opacity(0.7)
                    .hover(|style| style.opacity(1.0))
            )
    }
}
