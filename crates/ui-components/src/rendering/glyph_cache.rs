//! Glyph positioning and caching
//!
//! Phase 3.2: Text Rendering Pipeline

use std::collections::HashMap;

/// Cache for rendered glyphs (bitmaps)
pub struct GlyphCache {
    cache: HashMap<GlyphCacheKey, CachedGlyph>,
    /// Maximum cache size in bytes (~50MB)
    max_size: usize,
    /// Current cache size in bytes
    current_size: usize,
    /// LRU access order (most recent last)
    access_order: Vec<GlyphCacheKey>,
}

impl GlyphCache {
    pub fn new() -> Self {
        Self::with_capacity(50 * 1024 * 1024) // 50MB
    }

    pub fn with_capacity(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            current_size: 0,
            access_order: Vec::new(),
        }
    }

    /// Get or render a glyph from cache
    pub fn get_or_render(&mut self, key: GlyphCacheKey) -> Option<CachedGlyph> {
        // Update access order for LRU
        if let Some(pos) = self.access_order.iter().position(|k| k == &key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key.clone());

        if let Some(glyph) = self.cache.get(&key) {
            return Some(glyph.clone());
        }

        // In a real implementation, would rasterize the glyph here
        // For now, create a stub
        let metrics = GlyphMetrics {
            advance: 10.0,
            bearing_x: 0.0,
            bearing_y: 10.0,
            bitmap_width: 10,
            bitmap_height: 14,
        };

        let glyph = CachedGlyph {
            metrics,
            bitmap: vec![], // Empty bitmap for now
        };

        self.insert(key, glyph.clone());
        Some(glyph)
    }

    /// Insert a glyph into the cache with LRU eviction
    fn insert(&mut self, key: GlyphCacheKey, glyph: CachedGlyph) {
        let glyph_size = glyph.approximate_size();
        
        // Evict items if necessary to make room
        while self.current_size + glyph_size > self.max_size && !self.cache.is_empty() {
            if let Some(oldest_key) = self.access_order.first().cloned() {
                if let Some(evicted) = self.cache.remove(&oldest_key) {
                    self.current_size -= evicted.approximate_size();
                    self.access_order.remove(0);
                }
            }
        }

        let size = glyph.approximate_size();
        self.cache.insert(key, glyph);
        self.current_size += size;
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.access_order.clear();
        self.current_size = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entry_count: self.cache.len(),
            memory_used: self.current_size,
            memory_limit: self.max_size,
        }
    }

    /// Evict a percentage of the least recently used entries
    pub fn evict_lru(&mut self, percentage: f32) {
        let count_to_evict = ((self.cache.len() as f32) * percentage).ceil() as usize;
        
        for _ in 0..count_to_evict {
            if let Some(key) = self.access_order.first().cloned() {
                if let Some(glyph) = self.cache.remove(&key) {
                    self.current_size -= glyph.approximate_size();
                    self.access_order.remove(0);
                }
            }
        }
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Key for glyph cache lookups
#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub struct GlyphCacheKey {
    /// Glyph ID in font
    pub glyph_id: u32,
    /// Font identifier
    pub font_id: usize,
    /// Font size in pixels
    pub size: u32,
    /// Subpixel position X (0-63)
    pub subpixel_x: u8,
    /// Subpixel position Y (0-63)
    pub subpixel_y: u8,
}

impl GlyphCacheKey {
    pub fn new(glyph_id: u32, font_id: usize, size: u32) -> Self {
        Self {
            glyph_id,
            font_id,
            size,
            subpixel_x: 0,
            subpixel_y: 0,
        }
    }

    pub fn with_subpixel(mut self, x: u8, y: u8) -> Self {
        self.subpixel_x = x;
        self.subpixel_y = y;
        self
    }
}

/// Cached glyph data
#[derive(Clone)]
pub struct CachedGlyph {
    /// Glyph metrics and bitmap dimensions
    pub metrics: GlyphMetrics,
    /// Bitmap data (RGBA or grayscale)
    pub bitmap: Vec<u8>,
}

impl CachedGlyph {
    /// Estimate memory size in bytes
    fn approximate_size(&self) -> usize {
        std::mem::size_of::<CachedGlyph>() + self.bitmap.len()
    }
}

/// Glyph metrics including bitmap information
#[derive(Clone, Debug)]
pub struct GlyphMetrics {
    /// Horizontal advance width
    pub advance: f32,
    /// Left bearing (offset from origin to left edge)
    pub bearing_x: f32,
    /// Top bearing (offset from baseline to top)
    pub bearing_y: f32,
    /// Bitmap width
    pub bitmap_width: u32,
    /// Bitmap height
    pub bitmap_height: u32,
}

/// Cache statistics
pub struct CacheStats {
    /// Number of cached entries
    pub entry_count: usize,
    /// Memory used in bytes
    pub memory_used: usize,
    /// Memory limit in bytes
    pub memory_limit: usize,
}

impl CacheStats {
    /// Get cache usage as a percentage
    pub fn usage_percentage(&self) -> f32 {
        if self.memory_limit == 0 {
            0.0
        } else {
            (self.memory_used as f32 / self.memory_limit as f32) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_cache_creation() {
        let cache = GlyphCache::new();
        assert_eq!(cache.cache.len(), 0);
    }

    #[test]
    fn test_glyph_cache_key() {
        let key1 = GlyphCacheKey::new(1, 0, 14);
        let key2 = GlyphCacheKey::new(1, 0, 14);
        assert_eq!(key1, key2);

        let key3 = GlyphCacheKey::new(2, 0, 14);
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_glyph_cache_insert() {
        let mut cache = GlyphCache::with_capacity(1024 * 1024);
        let key = GlyphCacheKey::new(1, 0, 14);
        let glyph = CachedGlyph {
            metrics: GlyphMetrics {
                advance: 10.0,
                bearing_x: 0.0,
                bearing_y: 10.0,
                bitmap_width: 10,
                bitmap_height: 14,
            },
            bitmap: vec![],
        };

        cache.insert(key.clone(), glyph);
        assert_eq!(cache.cache.len(), 1);
    }

    #[test]
    fn test_cache_stats() {
        let cache = GlyphCache::with_capacity(10000);
        let stats = cache.stats();
        assert_eq!(stats.entry_count, 0);
        assert_eq!(stats.memory_limit, 10000);
    }

    #[test]
    fn test_lru_eviction() {
        let mut cache = GlyphCache::with_capacity(1024);
        
        // Insert some glyphs
        for i in 0..5 {
            let key = GlyphCacheKey::new(i, 0, 14);
            let glyph = CachedGlyph {
                metrics: GlyphMetrics {
                    advance: 10.0,
                    bearing_x: 0.0,
                    bearing_y: 10.0,
                    bitmap_width: 10,
                    bitmap_height: 14,
                },
                bitmap: vec![0; 100],
            };
            cache.insert(key, glyph);
        }

        let count_before = cache.cache.len();
        cache.evict_lru(0.2); // Evict 20%
        let count_after = cache.cache.len();
        
        assert!(count_after < count_before);
    }
}
