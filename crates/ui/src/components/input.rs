use crate::theme::Theme;
use gpui::*;
use gpui::prelude::FluentBuilder;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct Input {
    theme: Arc<RwLock<Theme>>,
    value: String,
    placeholder: String,
    is_password: bool,
    has_error: bool,
    error_message: Option<String>,
    on_change: Option<Arc<dyn Fn(String, &mut Context<Self>) + Send + Sync>>,
}

impl Input {
    pub fn new(theme: Arc<RwLock<Theme>>, placeholder: impl Into<String>) -> Self {
        Self {
            theme,
            value: String::new(),
            placeholder: placeholder.into(),
            is_password: false,
            has_error: false,
            error_message: None,
            on_change: None,
        }
    }

    pub fn password(mut self) -> Self {
        self.is_password = true;
        self
    }

    pub fn error(mut self, message: impl Into<String>) -> Self {
        self.has_error = true;
        self.error_message = Some(message.into());
        self
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
        where F: Fn(String, &mut Context<Self>) + Send + Sync + 'static
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn value(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: String) {
        self.value = value;
    }
}

impl Render for Input {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.ui.input_background);
        let border_color = if self.has_error {
            theme.parse_color(&theme.semantic.error)
        } else {
            theme.parse_color(&theme.ui.input_border)
        };
        let fg_color = theme.parse_color(&theme.foreground.editor);
        let placeholder_color = theme.parse_color(&theme.foreground.editor).opacity(0.5);

        div()
            .flex()
            .flex_col()
            .gap_1()
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
                    .child(
                        if self.value.is_empty() {
                            div().text_color(placeholder_color).child(self.placeholder.clone())
                        } else if self.is_password {
                            div().child("•".repeat(self.value.len()))
                        } else {
                            div().child(self.value.clone())
                        }
                    )
            )
            .when_some(self.error_message.clone(), |this, msg| {
                this.child(
                    div().text_xs().text_color(theme.parse_color(&theme.semantic.error)).child(msg)
                )
            })
    }
}
