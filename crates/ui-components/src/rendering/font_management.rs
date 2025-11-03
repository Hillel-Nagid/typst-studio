//! Font loading, caching, and fallback management
//!
//! Phase 3.2: Text Rendering Pipeline

use fontdb;
use std::sync::Arc;
use std::collections::HashMap;

/// Font manager for loading and caching fonts
pub struct FontManager {
    database: fontdb::Database,
    cache: HashMap<String, Arc<FontData>>,
}

impl FontManager {
    pub fn new() -> Self {
        let mut database = fontdb::Database::new();
        database.load_system_fonts();
        Self {
            database,
            cache: HashMap::new(),
        }
    }

    /// Load a font by family name and style
    pub fn load_font(&mut self, family: &str, weight: u16, italic: bool) -> Option<Arc<FontData>> {
        let cache_key = format!("{}:{}:{}", family, weight, italic);

        if let Some(cached) = self.cache.get(&cache_key) {
            return Some(cached.clone());
        }

        // Query the font database
        let query = fontdb::Query {
            families: &[fontdb::Family::Name(family)],
            weight: fontdb::Weight(weight),
            stretch: fontdb::Stretch::Normal,
            style: if italic { fontdb::Style::Italic } else { fontdb::Style::Normal },
        };

        let font_id = self.database.query(&query)?;
        
        // Load the font data
        let (font_data, _face_index) = self.database.face_source(font_id)?;
        
        let bytes = match font_data {
            fontdb::Source::Binary(data) => data.as_ref().as_ref().to_vec(),
            fontdb::Source::File(path) => {
                std::fs::read(path).ok()?
            }
            fontdb::Source::SharedFile(path, _) => {
                std::fs::read(path).ok()?
            }
        };

        let font = Arc::new(FontData {
            bytes,
            family: family.to_string(),
            weight,
            italic,
        });

        self.cache.insert(cache_key, font.clone());
        Some(font)
    }

    /// Get fallback font for a script
    pub fn get_fallback(&mut self, script: Script) -> Option<Arc<FontData>> {
        let fallback_family = match script {
            Script::Latin => "Courier New",
            Script::Arabic => "Arial",
            Script::Hebrew => "Arial",
            Script::Devanagari => "Noto Sans Devanagari",
            Script::CJK => "Noto Sans CJK SC",
            Script::Other => "monospace",
        };

        self.load_font(fallback_family, 400, false)
    }

    /// Clear the font cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

impl Default for FontManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Font data wrapper containing font bytes and metadata
pub struct FontData {
    /// Font file bytes
    pub bytes: Vec<u8>,
    /// Font family name
    pub family: String,
    /// Font weight (100-900)
    pub weight: u16,
    /// Whether font is italic
    pub italic: bool,
}

impl FontData {
    /// Get metrics for this font at a specific size
    pub fn metrics(&self, font_size: f32) -> Option<FontMetrics> {
        // Parse TTF to get metrics
        let face = ttf_parser::Face::parse(&self.bytes, 0).ok()?;

        let units_per_em = face.units_per_em() as f32;
        let scale = font_size / units_per_em;

        Some(FontMetrics {
            ascent: (face.ascender() as f32) * scale,
            descent: (face.descender() as f32) * scale,
            line_gap: (face.line_gap() as f32) * scale,
        })
    }
}

/// Font metrics
#[derive(Debug, Clone, Copy)]
pub struct FontMetrics {
    /// Ascender height
    pub ascent: f32,
    /// Descender depth (negative value)
    pub descent: f32,
    /// Line gap
    pub line_gap: f32,
}

impl FontMetrics {
    /// Calculate total line height
    pub fn line_height(&self) -> f32 {
        self.ascent - self.descent + self.line_gap
    }
}

/// Script identifier for font selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Script {
    /// Latin script (Roman, Cyrillic, Greek)
    Latin,
    /// Arabic script
    Arabic,
    /// Hebrew script
    Hebrew,
    /// Devanagari script
    Devanagari,
    /// CJK (Chinese, Japanese, Korean)
    CJK,
    /// Other scripts
    Other,
}

/// Font fallback chain configuration
pub struct FontFallbackChain {
    pub primary: String,
    pub fallbacks: Vec<String>,
}

impl FontFallbackChain {
    pub fn new(primary: String) -> Self {
        Self {
            primary,
            fallbacks: vec![],
        }
    }

    pub fn with_fallbacks(primary: String, fallbacks: Vec<String>) -> Self {
        Self { primary, fallbacks }
    }

    pub fn add_fallback(&mut self, font: String) {
        self.fallbacks.push(font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let fm = FontManager::new();
        assert_eq!(fm.cache_size(), 0);
    }

    #[test]
    fn test_font_fallback_chain() {
        let mut chain = FontFallbackChain::new("Arial".to_string());
        chain.add_fallback("Courier".to_string());
        chain.add_fallback("monospace".to_string());

        assert_eq!(chain.primary, "Arial");
        assert_eq!(chain.fallbacks.len(), 2);
    }

    #[test]
    fn test_script_enum() {
        let scripts = vec![
            Script::Latin,
            Script::Arabic,
            Script::Hebrew,
            Script::Devanagari,
            Script::CJK,
            Script::Other
        ];
        assert_eq!(scripts.len(), 6);
    }
}
