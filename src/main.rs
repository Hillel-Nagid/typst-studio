use anyhow::Result;
use gpui::*;
use tracing_subscriber;
use ui::TypstEditorApp;

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber
        ::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter
                ::from_default_env()
                .add_directive(tracing::Level::INFO.into())
        )
        .init();

    tracing::info!("Starting Typst Studio");

    // Initialize GPUI application
    Application::new().run(|cx: &mut AppContext| {
        // Create the main application window
        let app = TypstEditorApp::new(cx);
        app.open_main_window(cx);
    });

    Ok(())
}
