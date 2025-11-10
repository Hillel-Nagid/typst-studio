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

type ClickHandler = Arc<dyn Fn(&mut App) + Send + Sync + 'static>;
pub struct Button {
    label: SharedString,
    variant: ButtonVariant,
    theme: Arc<RwLock<Theme>>,
    on_click: Option<ClickHandler>,
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

    pub fn on_click<F>(mut self, handler: F) -> Self where F: Fn(&mut App) + Send + Sync + 'static {
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
        /*
        TODO: LOOK AS REFERENCE IN BUTTON.RS
        .when_some(self.on_click.filter(|_| clickable), |this, on_click| {
                this.on_click(move |event, window, cx| {
                    (on_click)(event, window, cx);
                })
            })
            .when_some(self.on_hover.filter(|_| hoverable), |this, on_hover| {
                this.on_hover(move |hovered, window, cx| {
                    (on_hover)(hovered, window, cx);
                })
            })
         */
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
                    move |_mouse_event, _window, cx| {
                        _window.prevent_default();
                        handler(cx)
                    } // TODO: fix handler type
                )
            })
            //     .px_4()
            //     .py_2()
            // .rounded_md()
            // .cursor_pointer()
            .child(self.label.clone())
    }
}
