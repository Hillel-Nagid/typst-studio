//! Viewport management for preview

use serde::{ Deserialize, Serialize };

/// Zoom level for preview
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZoomLevel {
    /// Fit width to viewport
    FitWidth,
    /// Fit entire page to viewport
    FitPage,
    /// Fit height to viewport
    FitHeight,
    /// Custom zoom percentage
    Custom(f32),
}

impl ZoomLevel {
    /// Convert to scale factor
    pub fn to_scale(
        &self,
        viewport_width: f32,
        viewport_height: f32,
        page_width: f32,
        page_height: f32
    ) -> f32 {
        match self {
            ZoomLevel::FitWidth => viewport_width / page_width,
            ZoomLevel::FitPage => {
                let width_scale = viewport_width / page_width;
                let height_scale = viewport_height / page_height;
                width_scale.min(height_scale)
            }
            ZoomLevel::FitHeight => viewport_height / page_height,
            ZoomLevel::Custom(scale) => *scale,
        }
    }
}

impl Default for ZoomLevel {
    fn default() -> Self {
        ZoomLevel::FitWidth
    }
}

/// Viewport for preview display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Width of viewport in pixels
    pub width: f32,
    /// Height of viewport in pixels
    pub height: f32,
    /// Scroll position X
    pub scroll_x: f32,
    /// Scroll position Y
    pub scroll_y: f32,
    /// Zoom level
    pub zoom: ZoomLevel,
}

impl Viewport {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            scroll_x: 0.0,
            scroll_y: 0.0,
            zoom: ZoomLevel::default(),
        }
    }

    /// Set viewport size
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Scroll to position
    pub fn scroll_to(&mut self, x: f32, y: f32) {
        self.scroll_x = x;
        self.scroll_y = y;
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, dx: f32, dy: f32) {
        self.scroll_x += dx;
        self.scroll_y += dy;
        self.clamp_scroll();
    }

    /// Set zoom level
    pub fn set_zoom(&mut self, zoom: ZoomLevel) {
        self.zoom = zoom;
    }

    /// Zoom in (increase by 25%)
    pub fn zoom_in(&mut self) {
        if let ZoomLevel::Custom(scale) = self.zoom {
            self.zoom = ZoomLevel::Custom((scale * 1.25).min(4.0));
        } else {
            self.zoom = ZoomLevel::Custom(1.25);
        }
    }

    /// Zoom out (decrease by 25%)
    pub fn zoom_out(&mut self) {
        if let ZoomLevel::Custom(scale) = self.zoom {
            self.zoom = ZoomLevel::Custom((scale / 1.25).max(0.1));
        } else {
            self.zoom = ZoomLevel::Custom(0.75);
        }
    }

    /// Get current scale factor for given page dimensions
    pub fn current_scale(&self, page_width: f32, page_height: f32) -> f32 {
        self.zoom.to_scale(self.width, self.height, page_width, page_height)
    }

    fn clamp_scroll(&mut self) {
        self.scroll_x = self.scroll_x.max(0.0);
        self.scroll_y = self.scroll_y.max(0.0);
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new(800.0, 600.0)
    }
}
