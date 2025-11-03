//! Cursor movement logic for bidirectional text

use crate::algorithm::BidiParagraph;
use crate::{ BidiError, Result };
use unicode_segmentation::UnicodeSegmentation;

/// Direction of cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    WordLeft,
    WordRight,
}

/// Position in text (line and column in grapheme clusters)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextPosition {
    pub line: usize,
    pub column: usize,
}

/// Cursor movement in bidirectional text
pub struct CursorMovement;

impl CursorMovement {
    /// Move cursor in visual direction with grapheme cluster awareness
    pub fn move_visual(
        paragraph: &BidiParagraph,
        logical_pos: usize,
        direction: MovementDirection
    ) -> Result<usize> {
        let text = paragraph.text();
        if text.is_empty() {
            return Ok(0);
        }

        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let grapheme_count = graphemes.len();

        match direction {
            MovementDirection::Left => {
                if logical_pos == 0 {
                    return Ok(0);
                }

                // Move by grapheme clusters
                let grapheme_pos = Self::char_to_grapheme_pos(text, logical_pos);
                if grapheme_pos == 0 {
                    Ok(0)
                } else {
                    let new_grapheme_pos = grapheme_pos - 1;
                    Ok(Self::grapheme_to_char_pos(text, new_grapheme_pos))
                }
            }

            MovementDirection::Right => {
                let grapheme_pos = Self::char_to_grapheme_pos(text, logical_pos);
                if grapheme_pos >= grapheme_count {
                    Ok(text.len())
                } else {
                    let new_grapheme_pos = grapheme_pos + 1;
                    Ok(Self::grapheme_to_char_pos(text, new_grapheme_pos))
                }
            }

            MovementDirection::Home => {
                // Move to start (respecting indentation on first press)
                let first_non_ws = text
                    .chars()
                    .position(|c| !c.is_whitespace())
                    .unwrap_or(0);

                if logical_pos != first_non_ws && first_non_ws > 0 {
                    Ok(first_non_ws)
                } else {
                    Ok(0)
                }
            }

            MovementDirection::End => {
                // Move to end
                Ok(text.len())
            }

            MovementDirection::WordLeft => { Self::move_word_boundary(text, logical_pos, false) }

            MovementDirection::WordRight => { Self::move_word_boundary(text, logical_pos, true) }

            _ =>
                Err(
                    BidiError::ProcessingError(
                        "Vertical movement not implemented for single paragraph".to_string()
                    )
                ),
        }
    }

    /// Move cursor in logical direction (for navigation like Ctrl+Left/Right)
    pub fn move_logical(text: &str, logical_pos: usize, forward: bool) -> usize {
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let grapheme_pos = Self::char_to_grapheme_pos(text, logical_pos);

        let new_grapheme_pos = if forward {
            (grapheme_pos + 1).min(graphemes.len())
        } else {
            grapheme_pos.saturating_sub(1)
        };

        Self::grapheme_to_char_pos(text, new_grapheme_pos)
    }

    /// Convert character position to grapheme position
    fn char_to_grapheme_pos(text: &str, char_pos: usize) -> usize {
        let mut grapheme_pos = 0;
        let mut char_count = 0;

        for grapheme in text.graphemes(true) {
            if char_count >= char_pos {
                break;
            }
            char_count += grapheme.chars().count();
            grapheme_pos += 1;
        }

        grapheme_pos
    }

    /// Convert grapheme position to character position
    fn grapheme_to_char_pos(text: &str, grapheme_pos: usize) -> usize {
        let mut char_pos = 0;

        for (i, grapheme) in text.graphemes(true).enumerate() {
            if i >= grapheme_pos {
                break;
            }
            char_pos += grapheme.chars().count();
        }

        char_pos
    }

    /// Move to word boundary
    fn move_word_boundary(text: &str, logical_pos: usize, forward: bool) -> Result<usize> {
        let graphemes: Vec<&str> = text.graphemes(true).collect();
        let grapheme_pos = Self::char_to_grapheme_pos(text, logical_pos);

        if forward {
            // Move forward to next word boundary
            let mut found_word = false;
            for i in grapheme_pos..graphemes.len() {
                let is_word_char = graphemes[i].chars().all(|c| (c.is_alphanumeric() || c == '_'));

                if !found_word && is_word_char {
                    found_word = true;
                } else if found_word && !is_word_char {
                    return Ok(Self::grapheme_to_char_pos(text, i));
                }
            }
            Ok(text.len())
        } else {
            // Move backward to previous word boundary
            let mut found_word = false;
            for i in (0..grapheme_pos).rev() {
                let is_word_char = graphemes[i].chars().all(|c| (c.is_alphanumeric() || c == '_'));

                if !found_word && is_word_char {
                    found_word = true;
                } else if found_word && !is_word_char {
                    return Ok(Self::grapheme_to_char_pos(text, i + 1));
                }
            }
            Ok(0)
        }
    }

    /// Move cursor vertically (requires multi-line context)
    pub fn move_vertical(
        lines: &[String],
        current_line: usize,
        current_column: usize,
        direction: MovementDirection,
        sticky_column: Option<usize>
    ) -> Result<TextPosition> {
        match direction {
            MovementDirection::Up => {
                if current_line == 0 {
                    Ok(TextPosition { line: 0, column: 0 })
                } else {
                    let target_line = current_line - 1;
                    let target_column = Self::adjust_column_for_line(
                        &lines[target_line],
                        sticky_column.unwrap_or(current_column)
                    );
                    Ok(TextPosition { line: target_line, column: target_column })
                }
            }

            MovementDirection::Down => {
                if current_line + 1 >= lines.len() {
                    let last_line = &lines[lines.len() - 1];
                    let last_column = last_line.graphemes(true).count();
                    Ok(TextPosition { line: lines.len() - 1, column: last_column })
                } else {
                    let target_line = current_line + 1;
                    let target_column = Self::adjust_column_for_line(
                        &lines[target_line],
                        sticky_column.unwrap_or(current_column)
                    );
                    Ok(TextPosition { line: target_line, column: target_column })
                }
            }

            _ => Err(BidiError::ProcessingError("Invalid vertical movement direction".to_string())),
        }
    }

    /// Adjust column to fit within target line
    fn adjust_column_for_line(line: &str, desired_column: usize) -> usize {
        let line_length = line.graphemes(true).count();
        desired_column.min(line_length)
    }
}
