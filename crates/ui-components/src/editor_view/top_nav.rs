//! Top navigation bar component
//!
//! Phase 3.1: Editor View Component Hierarchy - Top Navigation

use super::menu_bar::MenuBar;
use gpui::{
    div,
    Render,
    Window,
    Context,
    IntoElement,
    px,
    rgb,
    MouseButton,
    MouseDownEvent,
    Styled,
    InteractiveElement,
    ParentElement,
};
/// Top navigation bar component
pub struct TopNav {
    pub menu_bar: MenuBar,
}

impl TopNav {
    pub fn new() -> Self {
        Self {
            menu_bar: MenuBar::new(),
        }
    }
}

impl Default for TopNav {
    fn default() -> Self {
        Self::new()
    }
}
impl Render for TopNav {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .h(px(36.0))
            .bg(rgb(0x2d2d30))
            .flex()
            .items_center()
            .justify_between()
            .px(px(12.0))
            .on_mouse_down(
                MouseButton::Left,
                _cx.listener(|_this, _event: &MouseDownEvent, window: &mut Window, _cx| {
                    window.start_window_move();
                })
            )
            // Left section: Logo + Title
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .min_w(px(200.0))
                    // Logo
                    .child(div().child("▶").text_color(rgb(0x007acc)).text_size(px(16.0)))
                    // Title text
                    .child(
                        div()
                            .child("Typst Studio")
                            .text_color(rgb(0xcccccc))
                            .text_size(px(14.0))
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                    )
            )
            // Middle section: Menu Bar
            .child(
                div()
                    .flex()
                    .gap(px(0.0))
                    .flex_1()
                    .justify_center()
                    .children(
                        self.menu_bar.menus.iter().map(|menu| {
                            div()
                                .px(px(12.0))
                                .py(px(8.0))
                                .child(menu.title.clone())
                                .text_color(rgb(0xcccccc))
                                .text_size(px(13.0))
                                .hover(|style| style.bg(rgb(0x3e3e42)))
                                .on_mouse_down(
                                    MouseButton::Left,
                                    _cx.listener(
                                        |
                                            _this,
                                            _event: &MouseDownEvent,
                                            _window: &mut Window,
                                            _cx
                                        | {
                                            // Prevent window dragging when clicking menu items
                                        }
                                    )
                                )
                        })
                    )
            )
            // Right section: Window Controls
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(0.0))
                    .min_w(px(138.0))
                    // Minimize button
                    .child(
                        div()
                            .child("−")
                            .text_color(rgb(0xcccccc))
                            .text_size(px(18.0))
                            .px(px(12.0))
                            .py(px(6.0))
                            .hover(|style| style.bg(rgb(0x3e3e42)))
                            .on_mouse_down(
                                MouseButton::Left,
                                _cx.listener(
                                    |_this, _event: &MouseDownEvent, window: &mut Window, _cx| {
                                        window.minimize_window();
                                    }
                                )
                            )
                    )
                    // Maximize button
                    .child(
                        div()
                            .child("□")
                            .text_color(rgb(0xcccccc))
                            .text_size(px(14.0))
                            .px(px(12.0))
                            .py(px(6.0))
                            .hover(|style| style.bg(rgb(0x3e3e42)))
                            .on_mouse_down(
                                MouseButton::Left,
                                _cx.listener(
                                    |_this, _event: &MouseDownEvent, window: &mut Window, _cx| {
                                        window.toggle_fullscreen();
                                    }
                                )
                            )
                    )
                    // Close button
                    .child(
                        div()
                            .child("✕")
                            .text_color(rgb(0xffffff))
                            .text_size(px(14.0))
                            .bg(rgb(0xc42e1e))
                            .px(px(12.0))
                            .py(px(6.0))
                            .hover(|style| style.bg(rgb(0xe81123)))
                            .on_mouse_down(
                                MouseButton::Left,
                                _cx.listener(
                                    |_this, _event: &MouseDownEvent, _window: &mut Window, _cx| {
                                        // For now, we'll just print a message. Full window close would require different approach
                                        // The window typically closes when the last entity is removed
                                        tracing::info!("Close button clicked");
                                    }
                                )
                            )
                    )
            )
    }
}
