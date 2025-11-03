use gpui::{
    AppContext,
    Context,
    AnyElement,
    Entity,
    IntoElement,
    ParentElement,
    Render,
    Styled,
    Window,
    div,
    px,
    rgb,
};
use crate::panels::Panel;
pub struct Divider {}
impl Render for Divider {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().w(px(1.0)).h_full().bg(rgb(0x3e3e42))
    }
}
pub struct PanelsLayout {
    pub panels: Vec<Entity<Panel>>,
}

impl PanelsLayout {
    pub fn new() -> Self {
        Self { panels: Vec::new() }
    }
}
impl Render for PanelsLayout {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let panel_iter = self.panels
            .iter()
            .cloned()
            .map(|panel| panel.into_any_element());
        let divider_factory = || cx.new(|_| Divider {}).into_any_element();
        let elements_with_dividers = itertools::Itertools::intersperse_with(
            panel_iter,
            divider_factory
        );
        div().flex_1().flex().overflow_hidden().bg(rgb(0x1e1e1e)).children(elements_with_dividers)
    }
}
