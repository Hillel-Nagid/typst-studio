//! Line layout engine for rendering
//!
//! Phase 3.2: Text Rendering Pipeline

use crate::rendering::{ ShapedGlyph, BidiShapedText };
use bidi_text::Direction;

/// Line layout calculator
pub struct LineLayout {
    /// Line width limit
    pub width_limit: f32,
    /// Whether to enable word wrapping
    pub word_wrap: bool,
}

impl LineLayout {
    pub fn new(width_limit: f32) -> Self {
        Self {
            width_limit,
            word_wrap: false,
        }
    }

    /// Compute visual lines from logical line and shaped text
    pub fn compute_visual_lines(
        &self,
        logical_line: usize,
        shaped_glyphs: &[ShapedGlyph]
    ) -> Vec<VisualLine> {
        if !self.word_wrap || self.width_limit <= 0.0 {
            // Single visual line
            return vec![VisualLine {
                logical_line,
                visual_line_index: 0,
                char_range: (0, shaped_glyphs.len()),
                pixel_width: shaped_glyphs
                    .iter()
                    .map(|g| g.x_advance)
                    .sum(),
                baseline_y: 0.0,
                bidi_runs: vec![VisualTextRun {
                    start_glyph: 0,
                    end_glyph: shaped_glyphs.len(),
                    direction: Direction::LeftToRight,
                    x_offset: 0.0,
                }],
            }];
        }

        // Word wrap implementation
        let mut visual_lines = Vec::new();
        let mut current_width = 0.0;
        let mut line_start = 0;
        let mut visual_index = 0;

        for (i, glyph) in shaped_glyphs.iter().enumerate() {
            let glyph_width = glyph.x_advance;

            if current_width + glyph_width > self.width_limit && line_start < i {
                // Create visual line
                visual_lines.push(VisualLine {
                    logical_line,
                    visual_line_index: visual_index,
                    char_range: (line_start, i),
                    pixel_width: current_width,
                    baseline_y: 0.0,
                    bidi_runs: vec![VisualTextRun {
                        start_glyph: line_start,
                        end_glyph: i,
                        direction: Direction::LeftToRight,
                        x_offset: 0.0,
                    }],
                });

                current_width = glyph_width;
                line_start = i;
                visual_index += 1;
            } else {
                current_width += glyph_width;
            }
        }

        // Add remaining glyphs as final visual line
        if line_start < shaped_glyphs.len() {
            visual_lines.push(VisualLine {
                logical_line,
                visual_line_index: visual_index,
                char_range: (line_start, shaped_glyphs.len()),
                pixel_width: current_width,
                baseline_y: 0.0,
                bidi_runs: vec![VisualTextRun {
                    start_glyph: line_start,
                    end_glyph: shaped_glyphs.len(),
                    direction: Direction::LeftToRight,
                    x_offset: 0.0,
                }],
            });
        }

        visual_lines
    }

    /// Compute visual lines from bidirectional shaped text
    ///
    /// This method handles mixed RTL/LTR text by laying out each run
    /// according to its direction while maintaining proper visual order.
    pub fn compute_visual_lines_with_bidi(
        &self,
        logical_line: usize,
        bidi_text: &BidiShapedText
    ) -> Vec<VisualLine> {
        // Calculate x positions for each run based on direction
        let mut runs = Vec::new();
        let mut current_x = 0.0;

        for shaped_run in &bidi_text.runs {
            let run_width = shaped_run.shaped_text.width();

            let visual_run = VisualTextRun {
                start_glyph: 0,
                end_glyph: shaped_run.shaped_text.glyph_count(),
                direction: shaped_run.direction,
                x_offset: current_x,
            };

            runs.push((shaped_run, visual_run));
            current_x += run_width;
        }

        let total_width = current_x;

        // For RTL base direction, we need to reorder runs visually
        // but keep their logical content intact for rendering
        if bidi_text.base_direction == Direction::RightToLeft {
            // In RTL layout, runs are displayed from right to left
            // Calculate positions from right edge
            current_x = total_width;
            for (shaped_run, visual_run) in &mut runs {
                let run_width = shaped_run.shaped_text.width();
                visual_run.x_offset = current_x - run_width;
                current_x -= run_width;
            }
        }

        // Create visual line containing all runs
        let visual_line = VisualLine {
            logical_line,
            visual_line_index: 0,
            char_range: (0, bidi_text.full_text.len()),
            pixel_width: total_width,
            baseline_y: 0.0,
            bidi_runs: runs
                .into_iter()
                .map(|(_, run)| run)
                .collect(),
        };

        vec![visual_line]
    }
}

impl Default for LineLayout {
    fn default() -> Self {
        Self::new(0.0)
    }
}

/// Visual representation of a line
pub struct VisualLine {
    /// Source logical line number
    pub logical_line: usize,
    /// Index of this visual line (for wrapped lines)
    pub visual_line_index: usize,
    /// Range of glyphs in this visual line
    pub char_range: (usize, usize),
    /// Total pixel width of the line
    pub pixel_width: f32,
    /// Baseline Y position
    pub baseline_y: f32,
    /// Visual text runs (for bidirectional text)
    pub bidi_runs: Vec<VisualTextRun>,
}

/// A segment of text with consistent direction
pub struct VisualTextRun {
    /// Start glyph index
    pub start_glyph: usize,
    /// End glyph index
    pub end_glyph: usize,
    /// Text direction
    pub direction: Direction,
    /// X offset from line start
    pub x_offset: f32,
}
