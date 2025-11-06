use crate::config::Config;
use crate::document::{Document, DocumentId};
use crate::selection::MultiCursor;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

pub type WindowId = usize;
pub type WorkspaceId = usize;

#[derive(Clone)]
pub struct ApplicationState {
    pub windows: Vec<WindowId>,
    pub active_window: Option<WindowId>,
    pub workspaces: HashMap<WindowId, Arc<RwLock<WorkspaceState>>>,
    pub config: Arc<RwLock<Config>>,
    pub recent_files: Vec<PathBuf>,
}

impl ApplicationState {
    pub fn new(config: Config) -> Self {
        Self {
            windows: Vec::new(),
            active_window: None,
            workspaces: HashMap::new(),
            config: Arc::new(RwLock::new(config)),
            recent_files: Vec::new(),
        }
    }

    pub fn add_window(&mut self, window_id: WindowId, workspace: WorkspaceState) {
        self.windows.push(window_id);
        self.workspaces
            .insert(window_id, Arc::new(RwLock::new(workspace)));
        if self.active_window.is_none() {
            self.active_window = Some(window_id);
        }
    }

    pub fn get_active_workspace(&self) -> Option<Arc<RwLock<WorkspaceState>>> {
        self.active_window
            .and_then(|id| self.workspaces.get(&id))
            .cloned()
    }
}

#[derive(Clone)]
pub struct WorkspaceState {
    pub workspace_id: WorkspaceId,
    pub root: Option<PathBuf>,
    pub open_documents: HashMap<DocumentId, Arc<RwLock<EditorState>>>,
    pub active_document: Option<DocumentId>,
    pub sidebar_visible: bool,
    pub preview_visible: bool,
    pub console_visible: bool,
}

impl WorkspaceState {
    pub fn new(workspace_id: WorkspaceId) -> Self {
        Self {
            workspace_id,
            root: None,
            open_documents: HashMap::new(),
            active_document: None,
            sidebar_visible: true,
            preview_visible: true,
            console_visible: false,
        }
    }

    pub fn open_document(&mut self, document: Document) -> DocumentId {
        let id = document.id;
        let editor_state = EditorState::new(document);
        self.open_documents
            .insert(id, Arc::new(RwLock::new(editor_state)));
        self.active_document = Some(id);
        id
    }

    pub fn get_active_editor(&self) -> Option<Arc<RwLock<EditorState>>> {
        self.active_document
            .and_then(|id| self.open_documents.get(&id))
            .cloned()
    }

    pub fn close_document(&mut self, id: DocumentId) {
        self.open_documents.remove(&id);
        if self.active_document == Some(id) {
            self.active_document = self.open_documents.keys().next().copied();
        }
    }
}

#[derive(Clone)]
pub struct EditorState {
    pub document: Document,
    pub content: String,
    pub cursors: MultiCursor,
    pub scroll_offset: f32,
}

impl EditorState {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            content: String::new(),
            cursors: MultiCursor::default(),
            scroll_offset: 0.0,
        }
    }

    pub fn set_content(&mut self, content: String) {
        self.content = content;
        self.document.mark_dirty();
    }
}

