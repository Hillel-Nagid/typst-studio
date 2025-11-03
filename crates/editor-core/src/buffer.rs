//! Text buffer implementation using rope data structure

pub mod word_boundaries;

use crate::{ EditorError, Result, Version };
use crate::selection::Position;
use crate::operations::{ EditOperation, OperationType, UndoHistory };
use ropey::Rope;
use serde::{ Deserialize, Serialize };
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

pub use word_boundaries::WordBoundaryFinder;

/// Unique identifier for a buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferId(u64);

impl BufferId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

/// Line ending style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineEnding {
    /// Unix-style (LF)
    Lf,
    /// Windows-style (CRLF)
    Crlf,
    /// Classic Mac (CR) - rare
    Cr,
}

impl LineEnding {
    pub fn as_str(&self) -> &str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
            LineEnding::Cr => "\r",
        }
    }

    /// Detect line ending from text content
    pub fn detect(text: &str) -> Self {
        if text.contains("\r\n") {
            LineEnding::Crlf
        } else if text.contains('\n') {
            LineEnding::Lf
        } else if text.contains('\r') {
            LineEnding::Cr
        } else {
            // Default to platform-specific
            #[cfg(windows)]
            return LineEnding::Crlf;
            #[cfg(not(windows))]
            return LineEnding::Lf;
        }
    }
}

/// Immutable snapshot of a buffer at a point in time
#[derive(Clone)]
pub struct BufferSnapshot {
    rope: Rope,
    version: Version,
}

impl BufferSnapshot {
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    pub fn version(&self) -> Version {
        self.version
    }

    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    pub fn line(&self, line_idx: usize) -> Option<String> {
        if line_idx < self.len_lines() { Some(self.rope.line(line_idx).to_string()) } else { None }
    }
}

/// Metrics about the buffer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferMetrics {
    pub total_lines: usize,
    pub total_chars: usize,
    pub total_bytes: usize,
    pub longest_line_length: usize,
}

/// The main text buffer
pub struct Buffer {
    id: BufferId,
    rope: Rope,
    version: Version,
    file_path: Option<PathBuf>,
    line_ending: LineEnding,
    dirty: bool,
    read_only: bool,
    undo_history: UndoHistory,
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new(id: BufferId) -> Self {
        Self {
            id,
            rope: Rope::new(),
            version: Version::new(),
            file_path: None,
            line_ending: LineEnding::Lf,
            dirty: false,
            read_only: false,
            undo_history: UndoHistory::new(),
        }
    }

    /// Create a buffer from text content
    pub fn from_text(id: BufferId, text: &str) -> Self {
        let line_ending = LineEnding::detect(text);
        Self {
            id,
            rope: Rope::from_str(text),
            version: Version::new(),
            file_path: None,
            line_ending,
            dirty: false,
            read_only: false,
            undo_history: UndoHistory::new(),
        }
    }

    /// Create a buffer from a file path
    pub fn from_file(id: BufferId, path: PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(&path)?;
        let line_ending = LineEnding::detect(&content);
        Ok(Self {
            id,
            rope: Rope::from_str(&content),
            version: Version::new(),
            file_path: Some(path),
            line_ending,
            dirty: false,
            read_only: false,
            undo_history: UndoHistory::new(),
        })
    }

    /// Get buffer ID
    pub fn id(&self) -> BufferId {
        self.id
    }

    /// Get current version
    pub fn version(&self) -> Version {
        self.version
    }

    /// Get file path if any
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Set file path
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    /// Check if buffer has unsaved changes
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if buffer is read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Set read-only status
    pub fn set_read_only(&mut self, read_only: bool) {
        self.read_only = read_only;
    }

    /// Get the entire text content
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Get number of lines
    pub fn len_lines(&self) -> usize {
        self.rope.len_lines()
    }

    /// Get number of characters
    pub fn len_chars(&self) -> usize {
        self.rope.len_chars()
    }

    /// Get number of bytes
    pub fn len_bytes(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_chars() == 0
    }

    /// Get a specific line
    pub fn line(&self, line_idx: usize) -> Result<String> {
        if line_idx < self.len_lines() {
            Ok(self.rope.line(line_idx).to_string())
        } else {
            Err(EditorError::InvalidPosition {
                line: line_idx,
                column: 0,
            })
        }
    }

    /// Get line ending style
    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }

    /// Set line ending style
    pub fn set_line_ending(&mut self, ending: LineEnding) {
        self.line_ending = ending;
        self.dirty = true;
    }

    /// Convert position to character index
    pub fn position_to_char_idx(&self, pos: Position) -> Result<usize> {
        if pos.line >= self.len_lines() {
            return Err(EditorError::InvalidPosition {
                line: pos.line,
                column: pos.column,
            });
        }

        let line_start = self.rope.line_to_char(pos.line);
        let line = self.rope.line(pos.line);

        // Count grapheme clusters to respect unicode properly
        let graphemes: Vec<&str> = line.as_str().unwrap_or("").graphemes(true).collect();

        if pos.column > graphemes.len() {
            return Err(EditorError::InvalidPosition {
                line: pos.line,
                column: pos.column,
            });
        }

        let column_offset = graphemes[..pos.column]
            .iter()
            .map(|g| g.chars().count())
            .sum::<usize>();

        Ok(line_start + column_offset)
    }

    /// Convert character index to position
    pub fn char_idx_to_position(&self, idx: usize) -> Result<Position> {
        if idx > self.len_chars() {
            return Err(EditorError::InvalidPosition {
                line: 0,
                column: idx,
            });
        }

        let line = self.rope.char_to_line(idx);
        let line_start = self.rope.line_to_char(line);
        let line_content = self.rope.line(line);

        let char_offset = idx - line_start;
        let graphemes: Vec<&str> = line_content.as_str().unwrap_or("").graphemes(true).collect();

        let mut chars_counted = 0;
        let mut column = 0;
        for grapheme in graphemes {
            if chars_counted >= char_offset {
                break;
            }
            chars_counted += grapheme.chars().count();
            column += 1;
        }

        Ok(Position::new(line, column))
    }

    /// Insert text at a position
    pub fn insert(&mut self, pos: Position, text: &str) -> Result<()> {
        if self.read_only {
            return Err(EditorError::BufferError("Buffer is read-only".to_string()));
        }

        let char_idx = self.position_to_char_idx(pos)?;

        // Calculate cursor position after insertion
        let lines_added = text.matches('\n').count();
        let cursor_after = if lines_added > 0 {
            let last_line_len = text.lines().last().unwrap_or("").len();
            Position::new(pos.line + lines_added, last_line_len)
        } else {
            Position::new(pos.line, pos.column + text.len())
        };

        // Record operation for undo
        let operation = EditOperation::insert(pos, text.to_string(), cursor_after);
        self.undo_history.record_operation(operation);

        self.rope.insert(char_idx, text);
        self.version = self.version.next();
        self.dirty = true;
        Ok(())
    }

    /// Delete a range of text
    pub fn delete(&mut self, start: Position, end: Position) -> Result<String> {
        if self.read_only {
            return Err(EditorError::BufferError("Buffer is read-only".to_string()));
        }

        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        if start_idx > end_idx {
            return Err(
                EditorError::InvalidRange(
                    format!("Start position {:?} is after end position {:?}", start, end)
                )
            );
        }

        let deleted_text = self.rope.slice(start_idx..end_idx).to_string();

        // Record operation for undo
        let operation = EditOperation::delete(start, end, deleted_text.clone(), start);
        self.undo_history.record_operation(operation);

        self.rope.remove(start_idx..end_idx);
        self.version = self.version.next();
        self.dirty = true;
        Ok(deleted_text)
    }

    /// Replace a range of text
    pub fn replace(&mut self, start: Position, end: Position, text: &str) -> Result<String> {
        if self.read_only {
            return Err(EditorError::BufferError("Buffer is read-only".to_string()));
        }

        let start_idx = self.position_to_char_idx(start)?;
        let end_idx = self.position_to_char_idx(end)?;

        if start_idx > end_idx {
            return Err(
                EditorError::InvalidRange(
                    format!("Start position {:?} is after end position {:?}", start, end)
                )
            );
        }

        let deleted_text = self.rope.slice(start_idx..end_idx).to_string();

        // Calculate cursor position after replacement
        let lines_added = text.matches('\n').count();
        let cursor_after = if lines_added > 0 {
            let last_line_len = text.lines().last().unwrap_or("").len();
            Position::new(start.line + lines_added, last_line_len)
        } else {
            Position::new(start.line, start.column + text.len())
        };

        // Record operation for undo (as a single atomic operation)
        let operation = EditOperation::replace(
            start,
            end,
            deleted_text.clone(),
            text.to_string(),
            cursor_after
        );
        self.undo_history.record_operation(operation);

        self.rope.remove(start_idx..end_idx);
        self.rope.insert(start_idx, text);
        self.version = self.version.next();
        self.dirty = true;
        Ok(deleted_text)
    }

    /// Save buffer to file
    pub fn save(&mut self) -> std::io::Result<()> {
        if let Some(path) = &self.file_path {
            let content = self.text();
            std::fs::write(path, content)?;
            self.dirty = false;
            Ok(())
        } else {
            //TODO: should implement default fallback save path logic here using `self.save_as()`
            Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No file path set for buffer"))
        }
    }

    /// Save buffer to a specific file
    pub fn save_as(&mut self, path: PathBuf) -> std::io::Result<()> {
        let content = self.text();
        std::fs::write(&path, content)?;
        self.file_path = Some(path);
        self.dirty = false;
        Ok(())
    }

    /// Create an immutable snapshot
    pub fn snapshot(&self) -> BufferSnapshot {
        BufferSnapshot {
            rope: self.rope.clone(),
            version: self.version,
        }
    }

    /// Get buffer metrics
    pub fn metrics(&self) -> BufferMetrics {
        let longest_line = (0..self.len_lines())
            .map(|i| self.rope.line(i).len_chars())
            .max()
            .unwrap_or(0);

        BufferMetrics {
            total_lines: self.len_lines(),
            total_chars: self.len_chars(),
            total_bytes: self.len_bytes(),
            longest_line_length: longest_line,
        }
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<Position> {
        let group = self.undo_history.undo().ok_or(EditorError::UndoHistoryExhausted)?;

        // Apply operations in reverse
        for operation in group.operations.iter().rev() {
            match operation.op_type {
                OperationType::Insert => {
                    // Undo insert by deleting
                    if let Some(text) = &operation.inserted_text {
                        let end_pos = if text.contains('\n') {
                            let lines = text.matches('\n').count();
                            let last_line_len = text.lines().last().unwrap_or("").len();
                            Position::new(operation.start.line + lines, last_line_len)
                        } else {
                            Position::new(operation.start.line, operation.start.column + text.len())
                        };

                        let start_idx = self.position_to_char_idx(operation.start)?;
                        let end_idx = self.position_to_char_idx(end_pos)?;
                        self.rope.remove(start_idx..end_idx);
                    }
                }
                OperationType::Delete => {
                    // Undo delete by inserting
                    if let Some(text) = &operation.deleted_text {
                        let char_idx = self.position_to_char_idx(operation.start)?;
                        self.rope.insert(char_idx, text);
                    }
                }
                OperationType::Replace => {
                    // Undo replace by reversing the operation
                    if
                        let (Some(inserted), Some(deleted)) = (
                            &operation.inserted_text,
                            &operation.deleted_text,
                        )
                    {
                        let end_pos = if inserted.contains('\n') {
                            let lines = inserted.matches('\n').count();
                            let last_line_len = inserted.lines().last().unwrap_or("").len();
                            Position::new(operation.start.line + lines, last_line_len)
                        } else {
                            Position::new(
                                operation.start.line,
                                operation.start.column + inserted.len()
                            )
                        };

                        let start_idx = self.position_to_char_idx(operation.start)?;
                        let end_idx = self.position_to_char_idx(end_pos)?;
                        self.rope.remove(start_idx..end_idx);
                        self.rope.insert(start_idx, deleted);
                    }
                }
            }
        }

        self.version = self.version.next();
        self.dirty = true;

        // Return cursor position from first operation
        Ok(
            group.operations
                .first()
                .map(|op| op.cursor_before)
                .unwrap_or(Position::zero())
        )
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<Position> {
        let group = self.undo_history.redo().ok_or(EditorError::RedoHistoryExhausted)?;

        // Apply operations in forward order
        for operation in &group.operations {
            match operation.op_type {
                OperationType::Insert => {
                    if let Some(text) = &operation.inserted_text {
                        let char_idx = self.position_to_char_idx(operation.start)?;
                        self.rope.insert(char_idx, text);
                    }
                }
                OperationType::Delete => {
                    if let Some(end) = operation.end {
                        let start_idx = self.position_to_char_idx(operation.start)?;
                        let end_idx = self.position_to_char_idx(end)?;
                        self.rope.remove(start_idx..end_idx);
                    }
                }
                OperationType::Replace => {
                    if let (Some(end), Some(inserted)) = (operation.end, &operation.inserted_text) {
                        let start_idx = self.position_to_char_idx(operation.start)?;
                        let end_idx = self.position_to_char_idx(end)?;
                        self.rope.remove(start_idx..end_idx);
                        self.rope.insert(start_idx, inserted);
                    }
                }
            }
        }

        self.version = self.version.next();
        self.dirty = true;

        // Return cursor position from last operation
        Ok(
            group.operations
                .last()
                .map(|op| op.cursor_after)
                .unwrap_or(Position::zero())
        )
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.undo_history.can_redo()
    }

    /// Create a boundary in the undo history (force new undo group)
    pub fn create_undo_boundary(&mut self) {
        self.undo_history.create_boundary();
    }

    /// Clear undo/redo history
    pub fn clear_undo_history(&mut self) {
        self.undo_history.clear();
    }

    /// Delete previous grapheme cluster (backspace operation)
    pub fn backspace(&mut self, pos: Position) -> Result<Position> {
        if pos.line == 0 && pos.column == 0 {
            return Ok(pos); // Already at start
        }

        if pos.column == 0 {
            // At start of line - join with previous line
            let prev_line_idx = pos.line - 1;
            let prev_line = self.line(prev_line_idx)?;
            // Count graphemes, excluding trailing newline
            let prev_line_without_newline = prev_line.trim_end_matches(&['\n', '\r'][..]);
            let prev_line_len = prev_line_without_newline.graphemes(true).count();

            let start = Position::new(prev_line_idx, prev_line_len);
            let end = pos;
            self.delete(start, end)?;

            Ok(start)
        } else {
            // Delete previous grapheme
            let line_text = self.line(pos.line)?;
            let graphemes: Vec<&str> = line_text.graphemes(true).collect();

            if pos.column > 0 && pos.column <= graphemes.len() {
                let start = Position::new(pos.line, pos.column - 1);
                self.delete(start, pos)?;
                Ok(start)
            } else {
                Ok(pos)
            }
        }
    }

    /// Delete next grapheme cluster (delete key operation)
    pub fn delete_forward(&mut self, pos: Position) -> Result<Position> {
        let line_text = self.line(pos.line)?;
        let graphemes: Vec<&str> = line_text.graphemes(true).collect();

        if pos.column >= graphemes.len() {
            // At end of line - join with next line if exists
            if pos.line + 1 < self.len_lines() {
                let end = Position::new(pos.line + 1, 0);
                self.delete(pos, end)?;
            }
            Ok(pos)
        } else {
            // Delete next grapheme
            let end = Position::new(pos.line, pos.column + 1);
            self.delete(pos, end)?;
            Ok(pos)
        }
    }

    /// Get word boundaries in a line
    fn word_boundaries(&self, line_idx: usize) -> Result<Vec<usize>> {
        let line = self.line(line_idx)?;
        let mut boundaries = vec![0];

        let graphemes: Vec<&str> = line.graphemes(true).collect();
        let mut in_word = false;

        for (i, grapheme) in graphemes.iter().enumerate() {
            let is_word_char = grapheme.chars().all(|c| (c.is_alphanumeric() || c == '_'));

            if is_word_char && !in_word {
                boundaries.push(i);
                in_word = true;
            } else if !is_word_char && in_word {
                boundaries.push(i);
                in_word = false;
            }
        }

        if in_word {
            boundaries.push(graphemes.len());
        }

        Ok(boundaries)
    }

    /// Find next word boundary
    pub fn next_word_boundary(&self, pos: Position) -> Result<Position> {
        let boundaries = self.word_boundaries(pos.line)?;

        for &boundary in &boundaries {
            if boundary > pos.column {
                return Ok(Position::new(pos.line, boundary));
            }
        }

        // If at end of line, go to start of next line
        if pos.line + 1 < self.len_lines() {
            Ok(Position::new(pos.line + 1, 0))
        } else {
            Ok(pos)
        }
    }

    /// Find previous word boundary
    pub fn prev_word_boundary(&self, pos: Position) -> Result<Position> {
        let boundaries = self.word_boundaries(pos.line)?;

        for &boundary in boundaries.iter().rev() {
            if boundary < pos.column {
                return Ok(Position::new(pos.line, boundary));
            }
        }

        // If at start of line, go to end of previous line
        if pos.line > 0 {
            let prev_line = self.line(pos.line - 1)?;
            let col = prev_line.graphemes(true).count();
            Ok(Position::new(pos.line - 1, col))
        } else {
            Ok(pos)
        }
    }
}
