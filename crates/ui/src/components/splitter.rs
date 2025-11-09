use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub enum SplitDirection {
    Horizontal,
    Vertical,
}

pub struct Splitter {
    direction: SplitDirection,
    theme: Arc<RwLock<Theme>>,
    position: f32, // 0.0 to 1.0
}

impl Splitter {
    pub fn new(direction: SplitDirection, theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            direction,
            theme,
            position: 0.5,
        }
    }

    pub fn position(&self) -> f32 {
        self.position
    }

    pub fn set_position(&mut self, position: f32) {
        self.position = position.clamp(0.0, 1.0);
    }
}

impl Render for Splitter {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let divider_color = theme.parse_color(&theme.ui.divider);

        match self.direction {
            SplitDirection::Horizontal =>
                div().h_1().w_full().bg(divider_color).cursor_row_resize(),
            SplitDirection::Vertical => div().w_1().h_full().bg(divider_color).cursor_col_resize(),
        }
    }
}
