use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct MenuItem {
    pub label: String,
    pub shortcut: Option<String>,
    pub enabled: bool,
    pub action: Option<Arc<dyn Fn(&mut WindowContext) + Send + Sync>>,
    pub is_separator: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            enabled: true,
            action: None,
            is_separator: false,
        }
    }

    pub fn separator() -> Self {
        Self {
            label: String::new(),
            shortcut: None,
            enabled: true,
            action: None,
            is_separator: true,
        }
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut WindowContext) + Send + Sync + 'static,
    {
        self.action = Some(Arc::new(handler));
        self
    }
}

pub struct ContextMenu {
    theme: Arc<RwLock<Theme>>,
    items: Vec<MenuItem>,
    visible: bool,
}

impl ContextMenu {
    pub fn new(theme: Arc<RwLock<Theme>>) -> Self {
        Self {
            theme,
            items: Vec::new(),
            visible: false,
        }
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn show(&mut self) {
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }
}

impl Render for ContextMenu {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.panel);
        let fg_color = theme.parse_color(&theme.foreground.panel);
        let border_color = theme.parse_color(&theme.ui.border);
        let hover_color = theme.parse_color(&theme.ui.button_hover);

        if !self.visible {
            return div();
        }

        div()
            .absolute()
            .min_w_48()
            .bg(bg_color)
            .border_1()
            .border_color(border_color)
            .rounded_md()
            .shadow_lg()
            .z_index(9999)
            .py_1()
            .children(self.items.iter().map(|item| {
                if item.is_separator {
                    div()
                        .h_px()
                        .w_full()
                        .bg(border_color)
                        .my_1()
                } else {
                    let opacity = if item.enabled { 1.0 } else { 0.5 };
                    let action = item.action.clone();

                    div()
                        .w_full()
                        .px_3()
                        .py_2()
                        .flex()
                        .flex_row()
                        .justify_between()
                        .items_center()
                        .text_color(fg_color)
                        .opacity(opacity)
                        .when(item.enabled, |this| {
                            this.hover(|style| style.bg(hover_color))
                                .cursor_pointer()
                                .when_some(action, |this, handler| {
                                    this.on_click(move |_, cx| handler(cx))
                                })
                        })
                        .child(div().text_sm().child(item.label.clone()))
                        .when_some(item.shortcut.clone(), |this, shortcut| {
                            this.child(
                                div()
                                    .text_xs()
                                    .opacity(0.7)
                                    .child(shortcut),
                            )
                        })
                }
            }))
    }
}

