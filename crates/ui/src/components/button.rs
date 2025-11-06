use crate::theme::Theme;
use gpui::*;
use std::sync::Arc;
use parking_lot::RwLock;

#[derive(Clone)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Danger,
    Ghost,
}

pub struct Button {
    label: SharedString,
    variant: ButtonVariant,
    theme: Arc<RwLock<Theme>>,
    on_click: Option<Arc<dyn Fn(&mut WindowContext) + Send + Sync>>,
}

impl Button {
    pub fn new(
        label: impl Into<SharedString>,
        variant: ButtonVariant,
        theme: Arc<RwLock<Theme>>,
    ) -> Self {
        Self {
            label: label.into(),
            variant,
            theme,
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut WindowContext) + Send + Sync + 'static,
    {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl Render for Button {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.ui.button_background);
        let fg_color = theme.parse_color(&theme.foreground.editor);
        let on_click = self.on_click.clone();

        div()
            .px_4()
            .py_2()
            .rounded_md()
            .bg(bg_color)
            .text_color(fg_color)
            .cursor_pointer()
            .hover(|style| style.bg(theme.parse_color(&theme.ui.button_hover)))
            .active(|style| style.bg(theme.parse_color(&theme.ui.button_active)))
            .when_some(on_click, |this, handler| {
                this.on_click(move |_, cx| handler(cx))
            })
            .child(self.label.clone())
    }
}

