use crate::theme::Theme;
use gpui::*;
use gpui::prelude::FluentBuilder;
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
    on_click: Option<Arc<dyn Fn(&mut Context<Self>) + Send + Sync>>,
}

impl Button {
    pub fn new(
        label: impl Into<SharedString>,
        variant: ButtonVariant,
        theme: Arc<RwLock<Theme>>
    ) -> Self {
        Self {
            label: label.into(),
            variant,
            theme,
            on_click: None,
        }
    }

    pub fn on_click<F>(mut self, handler: F) -> Self
        where F: Fn(&mut Context<Self>) + Send + Sync + 'static
    {
        self.on_click = Some(Arc::new(handler));
        self
    }
}

impl Render for Button {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.ui.button_background);
        let fg_color = theme.parse_color(&theme.foreground.editor);
        let on_click = self.on_click.clone();

        div()
            .on_mouse_down(
                MouseButton::Left,
                |_mouse_event, _window, _cx| {
                    // TODO: add local style state, update it and then notify
                    // _cx.style(move |style| style.bg(theme.parse_color(&theme.ui.button_active)))
                }
            )
            .hover(|style| style.bg(theme.parse_color(&theme.ui.button_hover)))
            .bg(bg_color)
            .text_color(fg_color)
            .when_some(on_click, |this, handler| {
                this.on_mouse_down(
                    MouseButton::Left,
                    move |_mouse_event, _window, cx| handler(&mut cx) // TODO: fix handler type
                )
            })
            //     .px_4()
            //     .py_2()
            // .rounded_md()
            // .cursor_pointer()
            .child(self.label.clone())
    }
}
