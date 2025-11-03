//! Edit operations and undo/redo system

use crate::selection::Position;
use serde::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };

/// Type of edit operation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OperationType {
    Insert,
    Delete,
    Replace,
}

/// Represents a single edit operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditOperation {
    /// Type of operation
    pub op_type: OperationType,
    /// Start position of the operation
    pub start: Position,
    /// End position (for delete/replace)
    pub end: Option<Position>,
    /// Text inserted (for insert/replace)
    pub inserted_text: Option<String>,
    /// Text deleted (for delete/replace, used for undo)
    pub deleted_text: Option<String>,
    /// Cursor position before operation
    pub cursor_before: Position,
    /// Cursor position after operation
    pub cursor_after: Position,
    /// Timestamp of operation
    pub timestamp: u64,
}

impl EditOperation {
    pub fn insert(pos: Position, text: String, cursor_after: Position) -> Self {
        Self {
            op_type: OperationType::Insert,
            start: pos,
            end: None,
            inserted_text: Some(text),
            deleted_text: None,
            cursor_before: pos,
            cursor_after,
            timestamp: current_timestamp(),
        }
    }

    pub fn delete(
        start: Position,
        end: Position,
        deleted_text: String,
        cursor_after: Position
    ) -> Self {
        Self {
            op_type: OperationType::Delete,
            start,
            end: Some(end),
            inserted_text: None,
            deleted_text: Some(deleted_text),
            cursor_before: start,
            cursor_after,
            timestamp: current_timestamp(),
        }
    }

    pub fn replace(
        start: Position,
        end: Position,
        deleted_text: String,
        inserted_text: String,
        cursor_after: Position
    ) -> Self {
        Self {
            op_type: OperationType::Replace,
            start,
            end: Some(end),
            inserted_text: Some(inserted_text),
            deleted_text: Some(deleted_text),
            cursor_before: start,
            cursor_after,
            timestamp: current_timestamp(),
        }
    }

    /// Check if this operation can be merged with another
    pub fn can_merge_with(&self, other: &EditOperation) -> bool {
        // Only merge consecutive character insertions within 1 second
        if self.op_type != OperationType::Insert || other.op_type != OperationType::Insert {
            return false;
        }

        // Check timestamp proximity (within 1 second)
        if other.timestamp.saturating_sub(self.timestamp) > 1000 {
            return false;
        }

        // Check if operations are adjacent
        self.cursor_after == other.start
    }

    /// Merge another operation into this one
    pub fn merge(&mut self, other: EditOperation) {
        if let (Some(my_text), Some(other_text)) = (&mut self.inserted_text, other.inserted_text) {
            my_text.push_str(&other_text);
            self.cursor_after = other.cursor_after;
            self.timestamp = other.timestamp;
        }
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64
}

/// Group of operations that should be undone/redone together
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationGroup {
    pub operations: Vec<EditOperation>,
    pub timestamp: u64,
}

impl OperationGroup {
    pub fn new(operation: EditOperation) -> Self {
        let timestamp = operation.timestamp;
        Self {
            operations: vec![operation],
            timestamp,
        }
    }

    pub fn add_operation(&mut self, operation: EditOperation) {
        self.timestamp = operation.timestamp;
        self.operations.push(operation);
    }

    pub fn can_merge_with(&self, operation: &EditOperation) -> bool {
        if let Some(last) = self.operations.last() { last.can_merge_with(operation) } else { false }
    }
}

/// Manages undo/redo history
pub struct UndoHistory {
    undo_stack: Vec<OperationGroup>,
    redo_stack: Vec<OperationGroup>,
    current_group: Option<OperationGroup>,
    max_operations: usize,
    #[allow(dead_code)]
    max_memory_bytes: usize,
}

impl UndoHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_group: None,
            max_operations: 1000,
            max_memory_bytes: 10 * 1024 * 1024, // 10MB
        }
    }

    pub fn with_limits(max_operations: usize, max_memory_bytes: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_group: None,
            max_operations,
            max_memory_bytes,
        }
    }

    /// Record a new operation
    pub fn record_operation(&mut self, operation: EditOperation) {
        // Clear redo stack when new operation is recorded
        self.redo_stack.clear();

        if let Some(ref mut group) = self.current_group {
            if group.can_merge_with(&operation) {
                if let Some(last_op) = group.operations.last_mut() {
                    if last_op.can_merge_with(&operation) {
                        last_op.merge(operation);
                        return;
                    }
                }
            } else {
                // Start new group - finalize current one
                let finished_group = self.current_group.take().unwrap();
                self.undo_stack.push(finished_group);
            }
        }

        // Start or continue current group
        if self.current_group.is_none() {
            self.current_group = Some(OperationGroup::new(operation));
        } else {
            self.current_group.as_mut().unwrap().add_operation(operation);
        }

        // Enforce limits
        self.enforce_limits();
    }

    /// Force a boundary in the undo history
    pub fn create_boundary(&mut self) {
        if let Some(group) = self.current_group.take() {
            self.undo_stack.push(group);
        }
    }

    /// Get the next operation group to undo
    pub fn undo(&mut self) -> Option<OperationGroup> {
        // Finalize current group first
        self.create_boundary();

        if let Some(group) = self.undo_stack.pop() {
            self.redo_stack.push(group.clone());
            Some(group)
        } else {
            None
        }
    }

    /// Get the next operation group to redo
    pub fn redo(&mut self) -> Option<OperationGroup> {
        if let Some(group) = self.redo_stack.pop() {
            self.undo_stack.push(group.clone());
            Some(group)
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty() || self.current_group.is_some()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group = None;
    }

    fn enforce_limits(&mut self) {
        // Enforce operation count limit
        while self.undo_stack.len() > self.max_operations {
            self.undo_stack.remove(0);
        }

        // TODO: Implement memory limit enforcement
        // This would require calculating approximate memory usage
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}
