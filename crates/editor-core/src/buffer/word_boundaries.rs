//! Word boundary detection using Unicode Standard Annex #29

use unicode_segmentation::UnicodeSegmentation;

/// Word boundary type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryType {
    /// Start of a word
    WordStart,
    /// End of a word
    WordEnd,
    /// Whitespace boundary
    Whitespace,
    /// Punctuation boundary
    Punctuation,
}

/// Represents a boundary in text
#[derive(Debug, Clone)]
pub struct Boundary {
    /// Position in grapheme clusters
    pub position: usize,
    /// Type of boundary
    pub boundary_type: BoundaryType,
}

/// Word boundary finder using UAX #29
pub struct WordBoundaryFinder {
    graphemes: Vec<String>,
    boundaries: Vec<Boundary>,
}

impl WordBoundaryFinder {
    /// Create a new word boundary finder for the given text
    pub fn new(text: &str) -> Self {
        let graphemes: Vec<String> = text
            .graphemes(true)
            .map(|s| s.to_string())
            .collect();
        let boundaries = Self::find_boundaries(&graphemes);

        Self {
            graphemes,
            boundaries,
        }
    }

    /// Find all word boundaries in the grapheme sequence
    fn find_boundaries(graphemes: &[String]) -> Vec<Boundary> {
        if graphemes.is_empty() {
            return Vec::new();
        }

        let mut boundaries = Vec::new();
        boundaries.push(Boundary {
            position: 0,
            boundary_type: BoundaryType::WordStart,
        });

        let mut prev_was_word = false;
        let mut prev_was_whitespace = false;

        for (i, grapheme) in graphemes.iter().enumerate() {
            let is_word_char = Self::is_word_char(grapheme);
            let is_whitespace = grapheme.chars().all(char::is_whitespace);
            let is_punctuation = !is_word_char && !is_whitespace;

            // Detect word boundaries
            if i > 0 {
                if is_word_char && !prev_was_word {
                    boundaries.push(Boundary {
                        position: i,
                        boundary_type: BoundaryType::WordStart,
                    });
                } else if !is_word_char && prev_was_word {
                    boundaries.push(Boundary {
                        position: i,
                        boundary_type: BoundaryType::WordEnd,
                    });
                }

                if is_whitespace && !prev_was_whitespace {
                    boundaries.push(Boundary {
                        position: i,
                        boundary_type: BoundaryType::Whitespace,
                    });
                }

                if is_punctuation {
                    boundaries.push(Boundary {
                        position: i,
                        boundary_type: BoundaryType::Punctuation,
                    });
                }
            }

            prev_was_word = is_word_char;
            prev_was_whitespace = is_whitespace;
        }

        // Add end boundary
        boundaries.push(Boundary {
            position: graphemes.len(),
            boundary_type: if prev_was_word {
                BoundaryType::WordEnd
            } else {
                BoundaryType::Whitespace
            },
        });

        boundaries
    }

    /// Check if a grapheme is a word character
    fn is_word_char(grapheme: &str) -> bool {
        grapheme.chars().all(|c| { c.is_alphanumeric() || c == '_' || c == '\'' || c == '-' })
    }

    /// Get all boundaries
    pub fn boundaries(&self) -> &[Boundary] {
        &self.boundaries
    }

    /// Find the next word boundary after the given position
    pub fn next_word_boundary(&self, position: usize) -> Option<usize> {
        for boundary in &self.boundaries {
            if
                boundary.position > position &&
                (boundary.boundary_type == BoundaryType::WordStart ||
                    boundary.boundary_type == BoundaryType::WordEnd)
            {
                return Some(boundary.position);
            }
        }
        None
    }

    /// Find the previous word boundary before the given position
    pub fn prev_word_boundary(&self, position: usize) -> Option<usize> {
        for boundary in self.boundaries.iter().rev() {
            if
                boundary.position < position &&
                (boundary.boundary_type == BoundaryType::WordStart ||
                    boundary.boundary_type == BoundaryType::WordEnd)
            {
                return Some(boundary.position);
            }
        }
        None
    }

    /// Find the start of the word containing the given position
    pub fn word_start_at(&self, position: usize) -> usize {
        for boundary in self.boundaries.iter().rev() {
            if boundary.position <= position && boundary.boundary_type == BoundaryType::WordStart {
                return boundary.position;
            }
        }
        0
    }

    /// Find the end of the word containing the given position
    pub fn word_end_at(&self, position: usize) -> usize {
        for boundary in &self.boundaries {
            if boundary.position >= position && boundary.boundary_type == BoundaryType::WordEnd {
                return boundary.position;
            }
        }
        self.graphemes.len()
    }

    /// Get the word at the given position
    pub fn word_at(&self, position: usize) -> Option<String> {
        let start = self.word_start_at(position);
        let end = self.word_end_at(position);

        if start < end && end <= self.graphemes.len() {
            Some(self.graphemes[start..end].join(""))
        } else {
            None
        }
    }
}
