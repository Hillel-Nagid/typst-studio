//! Overlay components (autocomplete, hover info, parameter hints, etc.)
//!
//! Phase 3.1: Editor View Component Hierarchy

use editor_core::Position;

/// Overlay manager for popups and tooltips
pub struct Overlays {
    /// Active autocomplete popup
    pub autocomplete: Option<AutocompletePopup>,
    /// Active hover info
    pub hover: Option<HoverInfo>,
    /// Active parameter hints
    pub parameter_hints: Option<ParameterHints>,
    /// Active quick fixes menu
    pub quick_fixes: Option<QuickFixesMenu>,
}

impl Overlays {
    pub fn new() -> Self {
        Self {
            autocomplete: None,
            hover: None,
            parameter_hints: None,
            quick_fixes: None,
        }
    }

    /// Show autocomplete popup
    pub fn show_autocomplete(&mut self, popup: AutocompletePopup) {
        self.autocomplete = Some(popup);
    }

    /// Hide autocomplete popup
    pub fn hide_autocomplete(&mut self) {
        self.autocomplete = None;
    }

    /// Show hover info
    pub fn show_hover(&mut self, hover: HoverInfo) {
        self.hover = Some(hover);
    }

    /// Hide hover info
    pub fn hide_hover(&mut self) {
        self.hover = None;
    }

    /// Show parameter hints
    pub fn show_parameter_hints(&mut self, hints: ParameterHints) {
        self.parameter_hints = Some(hints);
    }

    /// Hide parameter hints
    pub fn hide_parameter_hints(&mut self) {
        self.parameter_hints = None;
    }

    /// Show quick fixes menu
    pub fn show_quick_fixes(&mut self, menu: QuickFixesMenu) {
        self.quick_fixes = Some(menu);
    }

    /// Hide quick fixes menu
    pub fn hide_quick_fixes(&mut self) {
        self.quick_fixes = None;
    }

    /// Hide all overlays
    pub fn hide_all(&mut self) {
        self.autocomplete = None;
        self.hover = None;
        self.parameter_hints = None;
        self.quick_fixes = None;
    }

    /// Check if any overlay is visible
    pub fn has_visible_overlay(&self) -> bool {
        self.autocomplete.is_some() ||
            self.hover.is_some() ||
            self.parameter_hints.is_some() ||
            self.quick_fixes.is_some()
    }
}

impl Default for Overlays {
    fn default() -> Self {
        Self::new()
    }
}

/// Autocomplete popup
#[derive(Debug, Clone)]
pub struct AutocompletePopup {
    /// Popup position
    pub position: Position,
    /// Completion items
    pub items: Vec<CompletionItem>,
    /// Selected item index
    pub selected: usize,
}

impl AutocompletePopup {
    pub fn new(position: Position, items: Vec<CompletionItem>) -> Self {
        Self {
            position,
            items,
            selected: 0,
        }
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    /// Select previous item
    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get selected item
    pub fn get_selected(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected)
    }
}

/// Completion item
#[derive(Debug, Clone)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

/// Completion kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    Function,
    Variable,
    Keyword,
    Constant,
    Type,
    Module,
}

/// Hover information tooltip
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Hover position
    pub position: Position,
    /// Markdown content
    pub content: String,
}

impl HoverInfo {
    pub fn new(position: Position, content: String) -> Self {
        Self { position, content }
    }
}

/// Parameter hints popup
#[derive(Debug, Clone)]
pub struct ParameterHints {
    /// Hints position
    pub position: Position,
    /// Signatures
    pub signatures: Vec<SignatureInfo>,
    /// Active signature index
    pub active_signature: usize,
    /// Active parameter index
    pub active_parameter: usize,
}

impl ParameterHints {
    pub fn new(position: Position, signatures: Vec<SignatureInfo>) -> Self {
        Self {
            position,
            signatures,
            active_signature: 0,
            active_parameter: 0,
        }
    }

    /// Get active signature
    pub fn get_active_signature(&self) -> Option<&SignatureInfo> {
        self.signatures.get(self.active_signature)
    }
}

/// Signature information
#[derive(Debug, Clone)]
pub struct SignatureInfo {
    pub label: String,
    pub parameters: Vec<ParameterInfo>,
    pub documentation: Option<String>,
}

/// Parameter information
#[derive(Debug, Clone)]
pub struct ParameterInfo {
    pub label: String,
    pub documentation: Option<String>,
}

/// Quick fixes menu
#[derive(Debug, Clone)]
pub struct QuickFixesMenu {
    /// Menu position
    pub position: Position,
    /// Available actions
    pub actions: Vec<CodeAction>,
    /// Selected action index
    pub selected: usize,
}

impl QuickFixesMenu {
    pub fn new(position: Position, actions: Vec<CodeAction>) -> Self {
        Self {
            position,
            actions,
            selected: 0,
        }
    }

    /// Select next action
    pub fn select_next(&mut self) {
        if !self.actions.is_empty() {
            self.selected = (self.selected + 1) % self.actions.len();
        }
    }

    /// Select previous action
    pub fn select_previous(&mut self) {
        if !self.actions.is_empty() {
            self.selected = if self.selected == 0 {
                self.actions.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    /// Get selected action
    pub fn get_selected(&self) -> Option<&CodeAction> {
        self.actions.get(self.selected)
    }
}

/// Code action
#[derive(Debug, Clone)]
pub struct CodeAction {
    pub title: String,
    pub kind: CodeActionKind,
}

/// Code action kind
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeActionKind {
    QuickFix,
    Refactor,
    SourceAction,
}

#[cfg(test)]
mod tests {}
