use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Tab {
    pub id: String,
    pub label: String,
    pub is_dirty: bool,
    pub is_active: bool,
    pub closeable: bool,
}

pub struct Tabs {
    theme: Arc<RwLock<Theme>>,
    tabs: Vec<Tab>,
    on_select: Option<Arc<dyn Fn(String, &mut WindowContext) + Send + Sync>>,
    on_close: Option<Arc<dyn Fn(String, &mut WindowContext) + Send + Sync>>,
}

impl Tabs {
    pub fn new(theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            theme,
            tabs: Vec::new(),
            on_select: None,
            on_close: None,
        }
    }

    pub fn add_tab(&mut self, tab: Tab) {
        self.tabs.push(tab);
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
        where F: Fn(String, &mut WindowContext) + Send + Sync + 'static
    {
        self.on_select = Some(Arc::new(handler));
        self
    }

    pub fn on_close<F>(mut self, handler: F) -> Self
        where F: Fn(String, &mut WindowContext) + Send + Sync + 'static
    {
        self.on_close = Some(Arc::new(handler));
        self
    }
}

impl Render for Tabs {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.panel);
        let border_color = theme.parse_color(&theme.ui.border);

        div()
            .h_10()
            .w_full()
            .bg(bg_color)
            .border_b_1()
            .border_color(border_color)
            .flex()
            .flex_row()
            .items_center()
            .overflow_x_scroll()
            .children(
                self.tabs.iter().map(|tab| {
                    let active_bg = if tab.is_active {
                        theme.parse_color(&theme.background.editor)
                    } else {
                        bg_color
                    };
                    let fg_color = theme.parse_color(&theme.foreground.panel);
                    let on_select = self.on_select.clone();
                    let on_close = self.on_close.clone();
                    let tab_id = tab.id.clone();
                    let tab_id_close = tab.id.clone();

                    div()
                        .h_full()
                        .px_4()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap_2()
                        .bg(active_bg)
                        .text_color(fg_color)
                        .border_r_1()
                        .border_color(border_color)
                        .cursor_pointer()
                        .when_some(on_select.clone(), |this, handler| {
                            this.on_click(move |_, cx| handler(tab_id.clone(), cx))
                        })
                        .when(tab.is_dirty, |this| { this.child(div().child("●").text_xs()) })
                        .child(div().child(tab.label.clone()))
                        .when(tab.closeable, |this| {
                            this.child(
                                div()
                                    .child("✕")
                                    .text_xs()
                                    .opacity(0.6)
                                    .hover(|style| style.opacity(1.0))
                                    .when_some(on_close, |this, handler| {
                                        this.on_click(move |event, cx| {
                                            event.stop_propagation();
                                            handler(tab_id_close.clone(), cx)
                                        })
                                    })
                            )
                        })
                })
            )
    }
}
