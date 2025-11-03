//! Text shaping using HarfBuzz (via rustybuzz)
//!
//! Phase 3.2: Text Rendering Pipeline

use crate::rendering::FontData;
use std::collections::HashMap;
use std::sync::Arc;
use bidi_text::{ BidiParagraph, Direction };

/// Text shaping service for complex script support
pub struct TextShaper {
    /// Cache for shaped text runs
    cache: HashMap<String, ShapedText>,
}

impl TextShaper {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Shape a text run with the given font and features
    pub fn shape(&mut self, text: &str, font_data: &Arc<FontData>) -> ShapedText {
        // Create cache key
        let cache_key = format!("{}:{:?}", text, font_data.family);

        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        // TODO: Implement proper rustybuzz integration
        // For now, create a simple stub that returns basic glyphs
        // This will be replaced with actual shaping once the API is confirmed
        let glyphs: Vec<ShapedGlyph> = text
            .chars()
            .enumerate()
            .map(|(i, ch)| ShapedGlyph {
                glyph_id: ch as u32,
                cluster: i as u32,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 8.0, // Approximate character width
                y_advance: 0.0,
            })
            .collect();

        let shaped = ShapedText { glyphs };

        // Cache the result
        self.cache.insert(cache_key, shaped.clone());
        shaped
    }

    /// Shape text with bidirectional support
    ///
    /// This method processes the text through the Unicode Bidirectional Algorithm (UAX #9)
    /// before shaping, ensuring correct visual ordering for mixed RTL/LTR text.
    pub fn shape_with_bidi(&mut self, text: &str, font_data: &Arc<FontData>) -> BidiShapedText {
        // Process text through bidi algorithm to get visual runs
        let para = BidiParagraph::new(text.to_string(), None);
        let visual_runs = para.visual_runs();

        let mut shaped_runs = Vec::new();

        // Shape each visual run separately, preserving its direction
        for run in visual_runs {
            let run_text = &text[run.logical_range.clone()];
            let shaped = self.shape(run_text, font_data);

            shaped_runs.push(BidiShapedRun {
                logical_range: run.logical_range,
                direction: run.direction,
                shaped_text: shaped,
                level: run.level,
            });
        }

        BidiShapedText {
            base_direction: para.base_direction(),
            runs: shaped_runs,
            full_text: text.to_string(),
        }
    }

    /// Clear the shape cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for TextShaper {
    fn default() -> Self {
        Self::new()
    }
}

/// Shaped text result containing positioned glyphs
#[derive(Clone, Debug)]
pub struct ShapedText {
    pub glyphs: Vec<ShapedGlyph>,
}

impl ShapedText {
    /// Calculate total width of shaped text
    pub fn width(&self) -> f32 {
        self.glyphs
            .iter()
            .map(|g| g.x_advance)
            .sum()
    }

    /// Get glyph at index
    pub fn glyph(&self, index: usize) -> Option<&ShapedGlyph> {
        self.glyphs.get(index)
    }

    /// Get number of glyphs
    pub fn glyph_count(&self) -> usize {
        self.glyphs.len()
    }
}

/// A single shaped glyph with positioning information
#[derive(Clone, Debug)]
pub struct ShapedGlyph {
    /// Glyph ID in the font
    pub glyph_id: u32,
    /// Cluster index (maps back to source character)
    pub cluster: u32,
    /// X offset from origin
    pub x_offset: f32,
    /// Y offset from origin
    pub y_offset: f32,
    /// Horizontal advance
    pub x_advance: f32,
    /// Vertical advance
    pub y_advance: f32,
}

impl ShapedGlyph {
    /// Get bounding box for this glyph (approximate)
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        (
            self.x_offset,
            self.y_offset,
            self.x_offset + self.x_advance,
            self.y_offset + self.y_advance,
        )
    }
}

/// Bidirectional shaped text - contains multiple shaped runs with direction information
#[derive(Clone, Debug)]
pub struct BidiShapedText {
    /// Base direction of the paragraph
    pub base_direction: Direction,
    /// Individual shaped runs, each with its direction and logical range
    pub runs: Vec<BidiShapedRun>,
    /// Original full text (for reference)
    pub full_text: String,
}

impl BidiShapedText {
    /// Calculate total width of all shaped runs
    pub fn width(&self) -> f32 {
        self.runs
            .iter()
            .map(|r| r.shaped_text.width())
            .sum()
    }

    /// Get number of runs
    pub fn run_count(&self) -> usize {
        self.runs.len()
    }
}

/// A shaped run with bidirectional information
#[derive(Clone, Debug)]
pub struct BidiShapedRun {
    /// Range in original (logical) text
    pub logical_range: std::ops::Range<usize>,
    /// Direction of this run
    pub direction: Direction,
    /// Shaped glyphs for this run
    pub shaped_text: ShapedText,
    /// Embedding level (from Unicode Bidi Algorithm)
    pub level: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_shaper_creation() {
        let shaper = TextShaper::new();
        assert_eq!(shaper.cache_size(), 0);
    }

    #[test]
    fn test_shaped_text_width() {
        let glyphs = vec![
            ShapedGlyph {
                glyph_id: 1,
                cluster: 0,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 10.0,
                y_advance: 0.0,
            },
            ShapedGlyph {
                glyph_id: 2,
                cluster: 1,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 12.0,
                y_advance: 0.0,
            }
        ];

        let shaped = ShapedText { glyphs };
        assert_eq!(shaped.width(), 22.0);
        assert_eq!(shaped.glyph_count(), 2);
    }

    #[test]
    fn test_bidi_shaped_text_creation() {
        let run1 = BidiShapedRun {
            logical_range: 0..5,
            direction: Direction::LeftToRight,
            shaped_text: ShapedText {
                glyphs: vec![ShapedGlyph {
                    glyph_id: 1,
                    cluster: 0,
                    x_offset: 0.0,
                    y_offset: 0.0,
                    x_advance: 8.0,
                    y_advance: 0.0,
                }],
            },
            level: 0,
        };

        let bidi_text = BidiShapedText {
            base_direction: Direction::LeftToRight,
            runs: vec![run1],
            full_text: "Hello".to_string(),
        };

        assert_eq!(bidi_text.base_direction, Direction::LeftToRight);
        assert_eq!(bidi_text.run_count(), 1);
        assert!(bidi_text.width() > 0.0);
    }
}
