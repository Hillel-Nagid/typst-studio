use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Tooltip {
    theme: Arc<RwLock<Theme>>,
    content: String,
    visible: bool,
}

impl Tooltip {
    pub fn new(theme: Arc<RwLock<Theme>>, content: impl Into<String>) -> Self {
        Self {
            theme,
            content: content.into(),
            visible: false,
        }
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Render for Tooltip {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.panel);
        let fg_color = theme.parse_color(&theme.foreground.panel);
        let border_color = theme.parse_color(&theme.ui.border);

        if !self.visible {
            return div();
        }

        div()
            .absolute()
            .px_2()
            .py_1()
            .bg(bg_color)
            .text_color(fg_color)
            .border_1()
            .border_color(border_color)
            .rounded_md()
            .text_xs()
            .max_w_64()
            .shadow_lg()
            .z_index(9999)
            .child(self.content.clone())
    }
}

