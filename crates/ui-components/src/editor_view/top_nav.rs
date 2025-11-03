//! Top navigation bar component
//!
//! Phase 3.1: Editor View Component Hierarchy - Top Navigation

use super::menu_bar::MenuBar;

/// Top navigation bar component
pub struct TopNav {
    pub menu_bar: MenuBar,
}

impl TopNav {
    pub fn new() -> Self {
        Self {
            menu_bar: MenuBar::new(),
        }
    }
}

impl Default for TopNav {
    fn default() -> Self {
        Self::new()
    }
}
