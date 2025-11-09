use crate::theme::Theme;
use crate::workspace::MainWindow;
use editor_core::{ ApplicationState, Config, WorkspaceState };
use gpui::*;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct TypstEditorApp {
    state: Arc<RwLock<ApplicationState>>,
    theme: Arc<RwLock<Theme>>,
}

impl TypstEditorApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        let config = Config::load();
        let theme = if config.appearance.theme == "light" { Theme::light() } else { Theme::dark() };

        let state = ApplicationState::new(config);

        Self {
            state: Arc::new(RwLock::new(state)),
            theme: Arc::new(RwLock::new(theme)),
        }
    }

    pub fn open_main_window(&self, cx: &mut Context<Self>) {
        let state = self.state.clone();
        let theme = self.theme.clone();

        let window_id = cx
            .open_window(WindowOptions::default(), |window, cx| {
                let workspace = WorkspaceState::new(0);
                state.write().add_window(0, workspace);

                cx.new(|cx| MainWindow::new(state.clone(), theme.clone(), cx))
            })
            .unwrap();
    }
}
