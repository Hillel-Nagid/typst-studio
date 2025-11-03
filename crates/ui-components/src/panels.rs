//! Panel components

use gpui::{ Render, Window, Context, IntoElement, div, rgb, Styled };

/// Panel type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanelType {
    Diagnostics,
    Search,
    Terminal,
    Output,
}

/// Panel component
pub struct Panel {
    /// Panel type
    panel_type: PanelType,
    /// Whether panel is visible
    visible: bool,
    /// Panel height
    height: f32,
}

impl Panel {
    pub fn new(panel_type: PanelType) -> Self {
        Self {
            panel_type,
            visible: false,
            height: 200.0,
        }
    }

    pub fn panel_type(&self) -> PanelType {
        self.panel_type
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_height(&mut self, height: f32) {
        self.height = height.max(100.0).min(600.0);
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

impl Render for Panel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex_1().flex().overflow_hidden().bg(rgb(0x1e1e1e))
    }
}
