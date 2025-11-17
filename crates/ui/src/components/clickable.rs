use gpui::{ App, IntoElement, InteractiveElement, MouseButton, Styled };
use std::sync::Arc;

pub type ClickHandler = Arc<dyn Fn(&mut App) + Send + Sync + 'static>;

pub struct Clickable<E: IntoElement> {
    on_click: Option<ClickHandler>,
    element: E,
}

impl<E: IntoElement> Clickable<E> {
    pub fn new(element: E) -> Self {
        Self {
            on_click: None,
            element,
        }
    }

    pub fn on_click(mut self, handler: ClickHandler) -> Self {
        self.on_click = Some(handler);
        self
    }
}

impl<E: IntoElement> IntoElement for Clickable<E> where E::Element: InteractiveElement + Styled {
    type Element = E::Element;

    fn into_element(self) -> Self::Element {
        let mut element = self.element.into_element();

        if let Some(handler) = self.on_click {
            element = element
                .on_mouse_down(MouseButton::Left, move |_mouse_event, _window, cx| {
                    handler(cx);
                })
                .cursor_pointer();
        }

        element
    }
}
