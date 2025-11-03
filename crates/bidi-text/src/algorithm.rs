//! Unicode Bidirectional Algorithm implementation

use unicode_bidi::BidiInfo as UnicodeBidiInfo;
use serde::{ Deserialize, Serialize };

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    LeftToRight,
    RightToLeft,
}

impl Direction {
    pub fn is_ltr(&self) -> bool {
        matches!(self, Direction::LeftToRight)
    }

    pub fn is_rtl(&self) -> bool {
        matches!(self, Direction::RightToLeft)
    }
}

/// Represents a visual run of text (consecutive characters at same embedding level)
#[derive(Debug, Clone)]
pub struct VisualRun {
    /// Range in the original (logical) text
    pub logical_range: std::ops::Range<usize>,
    /// Direction of this run
    pub direction: Direction,
    /// Embedding level
    pub level: u8,
}

/// Information about bidirectional text in a paragraph
#[derive(Clone)]
pub struct BidiInfo {
    /// The original text
    text: String,
    /// Base direction of the paragraph
    base_direction: Direction,
}

impl BidiInfo {
    /// Create bidi info from text
    pub fn new(text: &str, default_direction: Option<Direction>) -> Self {
        let info = UnicodeBidiInfo::new(text, None);

        // Determine base direction
        let base_direction = if let Some(dir) = default_direction {
            dir
        } else {
            // Auto-detect based on first strong directional character
            if !info.paragraphs.is_empty() && info.paragraphs[0].level.is_ltr() {
                Direction::LeftToRight
            } else {
                Direction::RightToLeft
            }
        };

        Self {
            text: text.to_string(),
            base_direction,
        }
    }

    /// Get the base direction
    pub fn base_direction(&self) -> Direction {
        self.base_direction
    }

    /// Get the visual runs for display
    pub fn visual_runs(&self, paragraph_range: std::ops::Range<usize>) -> Vec<VisualRun> {
        let info = UnicodeBidiInfo::new(&self.text, None);

        if info.paragraphs.is_empty() {
            return Vec::new();
        }

        let paragraph_info = &info.paragraphs[0];

        // Get the reordered levels
        let levels = info.reordered_levels(paragraph_info, paragraph_range.clone());

        let mut runs = Vec::new();
        let mut current_level = None;
        let mut current_start = paragraph_range.start;

        for (i, level) in levels.iter().enumerate() {
            let pos = paragraph_range.start + i;

            if Some(*level) != current_level {
                // Save previous run if exists
                if let Some(lvl) = current_level {
                    runs.push(VisualRun {
                        logical_range: current_start..pos,
                        direction: if lvl.is_ltr() {
                            Direction::LeftToRight
                        } else {
                            Direction::RightToLeft
                        },
                        level: lvl.number(),
                    });
                }
                current_level = Some(*level);
                current_start = pos;
            }
        }

        // Add final run
        if let Some(lvl) = current_level {
            runs.push(VisualRun {
                logical_range: current_start..paragraph_range.end,
                direction: if lvl.is_ltr() {
                    Direction::LeftToRight
                } else {
                    Direction::RightToLeft
                },
                level: lvl.number(),
            });
        }

        runs
    }

    /// Convert logical position to visual position
    pub fn logical_to_visual(&self, logical_pos: usize, text_len: usize) -> usize {
        let info = UnicodeBidiInfo::new(&self.text, None);

        if info.paragraphs.is_empty() {
            return logical_pos;
        }

        let paragraph_info = &info.paragraphs[0];
        let line_range = 0..text_len;

        // Get visual order - this returns (levels, runs)
        let (_levels, runs) = info.visual_runs(paragraph_info, line_range);

        // Find which run contains this position
        let mut visual_pos = 0;
        for run in runs {
            if run.contains(&logical_pos) {
                let offset = logical_pos - run.start;
                visual_pos += offset;
                break;
            }
            visual_pos += run.len();
        }

        visual_pos
    }

    /// Convert visual position to logical position
    pub fn visual_to_logical(&self, visual_pos: usize, text_len: usize) -> usize {
        let info = UnicodeBidiInfo::new(&self.text, None);

        if info.paragraphs.is_empty() {
            return visual_pos;
        }

        let paragraph_info = &info.paragraphs[0];
        let line_range = 0..text_len;

        // Get visual order
        let (_levels, runs) = info.visual_runs(paragraph_info, line_range);

        let mut accumulated = 0;
        for run in runs {
            let run_len = run.len();
            if visual_pos < accumulated + run_len {
                let offset = visual_pos - accumulated;
                return run.start + offset;
            }
            accumulated += run_len;
        }

        text_len
    }
}

/// Represents a paragraph with bidirectional text processing
#[derive(Clone)]
pub struct BidiParagraph {
    /// The text content
    text: String,
    /// Bidirectional information
    bidi_info: BidiInfo,
}

impl BidiParagraph {
    /// Create a new bidi paragraph
    pub fn new(text: String, default_direction: Option<Direction>) -> Self {
        let bidi_info = BidiInfo::new(&text, default_direction);
        Self { text, bidi_info }
    }

    /// Get the text
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Get the base direction
    pub fn base_direction(&self) -> Direction {
        self.bidi_info.base_direction()
    }

    /// Get visual runs for rendering
    pub fn visual_runs(&self) -> Vec<VisualRun> {
        self.bidi_info.visual_runs(0..self.text.len())
    }

    /// Convert logical to visual position
    pub fn logical_to_visual(&self, logical_pos: usize) -> usize {
        self.bidi_info.logical_to_visual(logical_pos, self.text.len())
    }

    /// Convert visual to logical position
    pub fn visual_to_logical(&self, visual_pos: usize) -> usize {
        self.bidi_info.visual_to_logical(visual_pos, self.text.len())
    }
}
