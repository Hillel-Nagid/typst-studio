//! Core editing functionality for Typst Studio
//!
//! This crate provides the fundamental data structures and operations
//! for text editing, including the text buffer, selections, and edit operations.

pub mod buffer;
pub mod selection;
pub mod operations;

// Re-export commonly used types
pub use buffer::{ Buffer, BufferId, BufferSnapshot, LineEnding };
pub use selection::{ Selection, Cursor, Position, Affinity, SelectionSet, Granularity };
pub use operations::{ EditOperation, OperationType, UndoHistory };

/// Version number for tracking buffer changes
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(u64);

impl Version {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new()
    }
}

/// Common error types for the editor core
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("Invalid position: line {line}, column {column}")] InvalidPosition {
        line: usize,
        column: usize,
    },

    #[error("Invalid range: {0}")] InvalidRange(String),

    #[error("Buffer operation failed: {0}")] BufferError(String),

    #[error("Undo history exhausted")]
    UndoHistoryExhausted,

    #[error("Redo history exhausted")]
    RedoHistoryExhausted,
}

pub type Result<T> = std::result::Result<T, EditorError>;
