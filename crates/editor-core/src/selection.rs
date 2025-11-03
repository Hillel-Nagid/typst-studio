//! Selection and cursor management

use serde::{ Deserialize, Serialize };

/// Represents a position in the text buffer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number in grapheme clusters (0-indexed)
    pub column: usize,
}

impl Position {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn zero() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.column.cmp(&other.column),
            ord => ord,
        }
    }
}

/// Cursor affinity for bidirectional text
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Affinity {
    /// Prefer the left/upstream side at boundaries
    Upstream,
    /// Prefer the right/downstream side at boundaries
    Downstream,
}

impl Default for Affinity {
    fn default() -> Self {
        Self::Downstream
    }
}

/// Represents a cursor position with affinity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    pub position: Position,
    pub affinity: Affinity,
    /// Sticky column for vertical movement
    pub sticky_column: Option<usize>,
}

impl Cursor {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            affinity: Affinity::default(),
            sticky_column: None,
        }
    }

    pub fn with_affinity(position: Position, affinity: Affinity) -> Self {
        Self {
            position,
            affinity,
            sticky_column: None,
        }
    }
}

/// Selection granularity for multi-level selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Granularity {
    /// Character-by-character selection
    Character,
    /// Word-by-word selection
    Word,
    /// Line-by-line selection
    Line,
    /// Block/rectangular selection
    Block,
}

/// Represents a text selection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Selection {
    /// The anchor point (where selection started)
    pub anchor: Position,
    /// The cursor/head point (current position)
    pub cursor: Cursor,
    /// Selection granularity
    pub granularity: Granularity,
}

impl Selection {
    pub fn new(anchor: Position, cursor: Position) -> Self {
        Self {
            anchor,
            cursor: Cursor::new(cursor),
            granularity: Granularity::Character,
        }
    }

    pub fn collapsed(position: Position) -> Self {
        Self {
            anchor: position,
            cursor: Cursor::new(position),
            granularity: Granularity::Character,
        }
    }

    /// Returns true if the selection is collapsed (no text selected)
    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.cursor.position
    }

    /// Returns the start and end positions of the selection (in order)
    pub fn range(&self) -> (Position, Position) {
        if self.anchor <= self.cursor.position {
            (self.anchor, self.cursor.position)
        } else {
            (self.cursor.position, self.anchor)
        }
    }

    /// Returns true if the cursor is at the end of the selection
    pub fn is_forward(&self) -> bool {
        self.cursor.position >= self.anchor
    }
}

/// Manages multiple selections (multi-cursor)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectionSet {
    /// The primary selection
    primary: usize,
    /// All selections (including primary)
    selections: Vec<Selection>,
}

impl SelectionSet {
    pub fn new(selection: Selection) -> Self {
        Self {
            primary: 0,
            selections: vec![selection],
        }
    }

    pub fn primary(&self) -> &Selection {
        &self.selections[self.primary]
    }

    pub fn primary_mut(&mut self) -> &mut Selection {
        &mut self.selections[self.primary]
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn add_selection(&mut self, selection: Selection) {
        self.selections.push(selection);
    }

    pub fn clear_secondary(&mut self) {
        if self.primary != 0 {
            let primary = self.selections.remove(self.primary);
            self.selections.clear();
            self.selections.push(primary);
            self.primary = 0;
        } else {
            self.selections.truncate(1);
        }
    }

    /// Merge overlapping selections
    pub fn merge_overlapping(&mut self) {
        if self.selections.len() <= 1 {
            return;
        }

        // Sort selections by range start
        let mut sorted_indices: Vec<usize> = (0..self.selections.len()).collect();
        sorted_indices.sort_by_key(|&i| self.selections[i].range().0);

        let mut merged = Vec::new();
        let mut current = self.selections[sorted_indices[0]].clone();

        for &idx in sorted_indices.iter().skip(1) {
            let sel = &self.selections[idx];
            let current_range = current.range();
            let sel_range = sel.range();

            if sel_range.0 <= current_range.1 {
                // Overlapping or adjacent - merge
                let new_end = current_range.1.max(sel_range.1);
                current = Selection::new(current_range.0, new_end);
            } else {
                // No overlap - push current and start new
                merged.push(current.clone());
                current = sel.clone();
            }
        }
        merged.push(current);

        // Find new primary index
        let old_primary = &self.selections[self.primary];
        self.primary = merged
            .iter()
            .position(|s| s.range() == old_primary.range())
            .unwrap_or(0);

        self.selections = merged;
    }
}

impl Default for SelectionSet {
    fn default() -> Self {
        Self::new(Selection::collapsed(Position::zero()))
    }
}
