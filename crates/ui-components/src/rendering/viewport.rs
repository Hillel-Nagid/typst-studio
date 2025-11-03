//! Viewport management for virtual scrolling
//!
//! Phase 3.2: Text Rendering Pipeline

use gpui::Point;
use gpui::Pixels;

/// Viewport for managing visible content region
#[derive(Clone)]
pub struct Viewport {
    /// Scroll offset in pixels
    pub scroll_offset: Point<Pixels>,
    /// Viewport bounds
    pub bounds: gpui::Bounds<Pixels>,
}

impl Viewport {
    pub fn new() -> Self {
        Self {
            scroll_offset: Point::default(),
            bounds: gpui::Bounds::new(
                gpui::Point::new(gpui::px(0.0), gpui::px(0.0)),
                gpui::Size::new(gpui::px(800.0), gpui::px(600.0))
            ),
        }
    }

    /// Set viewport bounds
    pub fn set_bounds(&mut self, bounds: gpui::Bounds<Pixels>) {
        self.bounds = bounds;
    }

    /// Get scroll position as f32
    pub fn scroll_x(&self) -> f32 {
        self.scroll_offset.x.into()
    }

    pub fn scroll_y(&self) -> f32 {
        self.scroll_offset.y.into()
    }

    /// Set scroll position
    pub fn set_scroll(&mut self, x: f32, y: f32) {
        self.scroll_offset = Point::new(gpui::px(x), gpui::px(y));
    }

    /// Scroll by delta
    pub fn scroll_by(&mut self, delta_x: f32, delta_y: f32) {
        let new_x = self.scroll_x() + delta_x;
        let new_y = self.scroll_y() + delta_y;
        self.set_scroll(new_x.max(0.0), new_y.max(0.0));
    }

    /// Calculate visible line range
    pub fn visible_line_range(&self, line_height: f32) -> (usize, usize) {
        if line_height <= 0.0 {
            return (0, 0);
        }

        let scroll_y = self.scroll_y();
        let bounds_height: f32 = self.bounds.size.height.into();

        let first_line = (scroll_y / line_height).floor() as usize;
        let last_line = ((scroll_y + bounds_height) / line_height).ceil() as usize;

        // Add padding for smooth scrolling
        let padded_first = first_line.saturating_sub(3);
        let padded_last = last_line + 3;

        (padded_first, padded_last)
    }

    /// Ensure position is visible by scrolling if necessary
    pub fn ensure_visible(&mut self, y: f32, height: f32, _line_height: f32) {
        let bounds_height: f32 = self.bounds.size.height.into();
        let scroll_y = self.scroll_y();

        if y < scroll_y {
            // Position above viewport
            self.set_scroll(self.scroll_x(), y);
        } else if y + height > scroll_y + bounds_height {
            // Position below viewport
            let new_scroll_y = (y + height - bounds_height).max(0.0);
            self.set_scroll(self.scroll_x(), new_scroll_y);
        }
    }

    /// Get viewport height
    pub fn height(&self) -> f32 {
        self.bounds.size.height.into()
    }

    /// Get viewport width
    pub fn width(&self) -> f32 {
        self.bounds.size.width.into()
    }
}

impl Default for Viewport {
    fn default() -> Self {
        Self::new()
    }
}

/// Scroll anchor for maintaining position during edits
pub struct ScrollAnchor {
    /// Anchored line number
    pub line: usize,
    /// Offset within that line (pixels from top)
    pub offset: f32,
}

impl ScrollAnchor {
    pub fn new(line: usize, offset: f32) -> Self {
        Self { line, offset }
    }

    /// Create anchor from current scroll position
    pub fn from_scroll(scroll_y: f32, line_height: f32) -> Self {
        let line = (scroll_y / line_height).floor() as usize;
        let offset = scroll_y - (line as f32) * line_height;
        Self { line, offset }
    }

    /// Calculate scroll position from anchor
    pub fn to_scroll(&self, line_height: f32) -> f32 {
        (self.line as f32) * line_height + self.offset
    }
}
