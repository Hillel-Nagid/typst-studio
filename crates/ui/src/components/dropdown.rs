use crate::theme::Theme;
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct DropdownOption {
    pub value: String,
    pub label: String,
}

pub struct Dropdown {
    theme: Arc<RwLock<Theme>>,
    options: Vec<DropdownOption>,
    selected_index: usize,
    is_open: bool,
    on_select: Option<Arc<dyn Fn(String, &mut WindowContext) + Send + Sync>>,
}

impl Dropdown {
    pub fn new(theme: Arc<RwLock<Theme>>, options: Vec<DropdownOption>) -> Self {
        Self {
            theme,
            options,
            selected_index: 0,
            is_open: false,
            on_select: None,
        }
    }

    pub fn on_select<F>(mut self, handler: F) -> Self
    where
        F: Fn(String, &mut WindowContext) + Send + Sync + 'static,
    {
        self.on_select = Some(Arc::new(handler));
        self
    }

    pub fn selected_value(&self) -> Option<&str> {
        self.options.get(self.selected_index).map(|o| o.value.as_str())
    }

    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }
}

impl Render for Dropdown {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.ui.input_background);
        let border_color = theme.parse_color(&theme.ui.input_border);
        let fg_color = theme.parse_color(&theme.foreground.editor);
        let hover_color = theme.parse_color(&theme.ui.button_hover);

        let selected_label = self
            .options
            .get(self.selected_index)
            .map(|o| o.label.clone())
            .unwrap_or_default();

        div()
            .relative()
            .w_full()
            .child(
                div()
                    .w_full()
                    .px_3()
                    .py_2()
                    .bg(bg_color)
                    .border_1()
                    .border_color(border_color)
                    .rounded_md()
                    .text_color(fg_color)
                    .cursor_pointer()
                    .flex()
                    .flex_row()
                    .justify_between()
                    .items_center()
                    .child(div().child(selected_label))
                    .child(div().child(if self.is_open { "▲" } else { "▼" }).text_xs()),
            )
            .when(self.is_open, |this| {
                this.child(
                    div()
                        .absolute()
                        .top_12()
                        .left_0()
                        .w_full()
                        .max_h_64()
                        .overflow_y_scroll()
                        .bg(bg_color)
                        .border_1()
                        .border_color(border_color)
                        .rounded_md()
                        .shadow_lg()
                        .z_index(1000)
                        .children(self.options.iter().enumerate().map(|(idx, option)| {
                            let is_selected = idx == self.selected_index;
                            let on_select = self.on_select.clone();
                            let value = option.value.clone();

                            div()
                                .w_full()
                                .px_3()
                                .py_2()
                                .text_color(fg_color)
                                .when(is_selected, |this| {
                                    this.bg(hover_color)
                                })
                                .hover(|style| style.bg(hover_color))
                                .cursor_pointer()
                                .when_some(on_select, |this, handler| {
                                    this.on_click(move |_, cx| handler(value.clone(), cx))
                                })
                                .child(option.label.clone())
                        })),
                )
            })
    }
}

