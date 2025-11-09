use crate::theme::Theme;
use editor_core::ApplicationState;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

enum SidebarView {
    FileExplorer,
    Outline,
}

pub struct Sidebar {
    theme: Arc<RwLock<Theme>>,
    state: Arc<RwLock<ApplicationState>>,
    active_view: SidebarView,
}

impl Sidebar {
    pub fn new(
        theme: Arc<RwLock<Theme>>,
        state: Arc<RwLock<ApplicationState>>,
        cx: &mut Context<Self>
    ) -> Self {
        Self {
            theme,
            state,
            active_view: SidebarView::FileExplorer,
        }
    }
}

impl Render for Sidebar {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.sidebar);
        let fg_color = theme.parse_color(&theme.foreground.sidebar);

        div()
            .w_64()
            .h_full()
            .bg(bg_color)
            .text_color(fg_color)
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(theme.parse_color(&theme.ui.border))
            // Sidebar tabs
            .child(
                div()
                    .h_10()
                    .w_full()
                    .flex()
                    .flex_row()
                    .items_center()
                    .px_2()
                    .gap_2()
                    .border_b_1()
                    .border_color(theme.parse_color(&theme.ui.border))
                    .child(div().text_sm().font_weight(FontWeight::BOLD).child("Explorer"))
            )
            // Content
            .child(
                div()
                    .flex_1()
                    .p_2()
                    //TODO: add scroll on overflow
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .text_sm()
                            .child(div().child("📄 untitled.typ"))
                            .child(div().opacity(0.6).child("No files open"))
                    )
            )
    }
}
