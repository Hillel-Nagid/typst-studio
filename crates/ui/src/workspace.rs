use crate::components::StatusBar;
use crate::console::ConsolePanel;
use crate::editor::EditorPanel;
use crate::navbar::NavBar;
use crate::preview_pane::PreviewPane;
use crate::sidebar::Sidebar;
use crate::theme::Theme;
use editor_core::{ ApplicationState, Document };
use gpui::*;
use gpui::prelude::FluentBuilder;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct MainWindow {
    state: Arc<RwLock<ApplicationState>>,
    theme: Arc<RwLock<Theme>>,
    navbar: Entity<NavBar>,
    sidebar: Entity<Sidebar>,
    editor: Entity<EditorPanel>,
    preview: Entity<PreviewPane>,
    console: Entity<ConsolePanel>,
    status_bar: Entity<StatusBar>,
}

impl MainWindow {
    pub fn new(
        state: Arc<RwLock<ApplicationState>>,
        theme: Arc<RwLock<Theme>>,
        cx: &mut Context<Self>
    ) -> Self {
        let navbar = cx.new(|cx| NavBar::new(theme.clone(), cx));
        let sidebar = cx.new(|cx| Sidebar::new(theme.clone(), state.clone(), cx));
        let editor = cx.new(|cx| EditorPanel::new(theme.clone(), state.clone(), cx));
        let preview = cx.new(|cx| PreviewPane::new(theme.clone(), cx));
        let console = cx.new(|cx| ConsolePanel::new(theme.clone(), cx));
        let status_bar = cx.new(|_cx| StatusBar::new(theme.clone()));

        // Open a default document
        if let Some(workspace) = state.read().get_active_workspace() {
            let mut workspace = workspace.write();
            let doc = Document::new(None);
            workspace.open_document(doc);
        }

        Self {
            state,
            theme,
            navbar,
            sidebar,
            editor,
            preview,
            console,
            status_bar,
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme.read();
        let bg_color = theme.parse_color(&theme.background.editor);

        let workspace_state = self.state.read();
        let active_workspace = workspace_state.get_active_workspace();
        let (sidebar_visible, preview_visible, console_visible) = if
            let Some(ws) = active_workspace
        {
            let ws = ws.read();
            (ws.sidebar_visible, ws.preview_visible, ws.console_visible)
        } else {
            (true, true, false)
        };

        div()
            .size_full()
            .bg(bg_color)
            .flex()
            .flex_col()
            .child(self.navbar.clone())
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_row()
                    .when(sidebar_visible, |this| this.child(self.sidebar.clone()))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_row()
                                    .child(self.editor.clone())
                                    .when(preview_visible, |this| this.child(self.preview.clone()))
                            )
                            .when(console_visible, |this| this.child(self.console.clone()))
                    )
            )
            .child(self.status_bar.clone())
    }
}
