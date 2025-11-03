//! Visual layout of bidirectional text

use crate::algorithm::{ Direction, VisualRun as BidiVisualRun };
use serde::{ Deserialize, Serialize };

/// Represents a visual run of text with rendering information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualRun {
    /// The text content
    pub text: String,
    /// Direction of this run
    pub direction: Direction,
    /// Horizontal offset in pixels
    pub x_offset: f32,
    /// Width in pixels
    pub width: f32,
}

impl VisualRun {
    pub fn new(text: String, direction: Direction) -> Self {
        Self {
            text,
            direction,
            x_offset: 0.0,
            width: 0.0,
        }
    }
}

/// Represents a visual line (may be a wrapped portion of a logical line)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualLine {
    /// Index of the logical line
    pub logical_line: usize,
    /// Index within wrapped line (0 for first, 1 for second wrap, etc.)
    pub visual_line_index: usize,
    /// Character range from the logical line
    pub char_range: std::ops::Range<usize>,
    /// Total width in pixels
    pub pixel_width: f32,
    /// Baseline Y position
    pub baseline_y: f32,
    /// Visual runs that make up this line
    pub bidi_runs: Vec<VisualRun>,
}

impl VisualLine {
    pub fn new(
        logical_line: usize,
        visual_line_index: usize,
        char_range: std::ops::Range<usize>
    ) -> Self {
        Self {
            logical_line,
            visual_line_index,
            char_range,
            pixel_width: 0.0,
            baseline_y: 0.0,
            bidi_runs: Vec::new(),
        }
    }

    /// Add a visual run to this line
    pub fn add_run(&mut self, run: VisualRun) {
        self.pixel_width += run.width;
        self.bidi_runs.push(run);
    }

    /// Calculate the total width
    pub fn calculate_width(&mut self) {
        self.pixel_width = self.bidi_runs
            .iter()
            .map(|r| r.width)
            .sum();
    }
}

/// Layout engine for bidirectional text
pub struct BidiLayoutEngine {
    /// Line height in pixels
    line_height: f32,
    /// Font size in pixels
    font_size: f32,
}

impl BidiLayoutEngine {
    pub fn new(font_size: f32, line_height: f32) -> Self {
        Self {
            line_height,
            font_size,
        }
    }

    /// Calculate visual layout for a line of text
    pub fn layout_line(
        &self,
        logical_line: usize,
        text: &str,
        bidi_runs: Vec<BidiVisualRun>
    ) -> VisualLine {
        let mut visual_line = VisualLine::new(logical_line, 0, 0..text.len());

        let mut x_offset = 0.0;
        for run in bidi_runs {
            let run_text = &text[run.logical_range.clone()];

            // Simple width calculation (would use proper text shaping in real impl)
            let width = (run_text.len() as f32) * self.font_size * 0.6;

            let mut visual_run = VisualRun::new(run_text.to_string(), run.direction);
            visual_run.x_offset = x_offset;
            visual_run.width = width;

            visual_line.add_run(visual_run);
            x_offset += width;
        }

        visual_line.calculate_width();
        visual_line
    }
}
