//! Typst Studio - Main Application Entry Point

#![recursion_limit = "512"]

mod state;
mod app;

use gpui::{
    App,
    AppContext,
    Application,
    Bounds,
    TitlebarOptions,
    WindowBounds,
    WindowOptions,
    px,
    size,
};
use app::TypstEditorWindow;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).with_target(false).init();

    tracing::info!("Starting Typst Studio");

    // Create and run GPUI application
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1400.0), px(900.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Maximized(bounds)),
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    ..Default::default()
                }),

                ..Default::default()
            },
            |_, cx| { cx.new(|cx| TypstEditorWindow::new(cx)) }
        ).unwrap();

        tracing::info!("Editor window opened");
    });
}
