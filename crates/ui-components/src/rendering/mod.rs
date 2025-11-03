//! Rendering subsystem for text and graphics
//!
//! Phase 3.2: Text Rendering Pipeline

pub mod text_shaping;
pub mod font_management;
pub mod glyph_cache;
pub mod line_layout;
pub mod viewport;

pub use text_shaping::{ TextShaper, ShapedText, ShapedGlyph, BidiShapedText, BidiShapedRun };
pub use font_management::{ FontManager, FontData, Script };
pub use glyph_cache::{ GlyphCache, GlyphCacheKey };
pub use line_layout::{ LineLayout, VisualLine, VisualTextRun };
pub use viewport::{ Viewport, ScrollAnchor };
