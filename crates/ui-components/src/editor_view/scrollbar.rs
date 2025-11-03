//! Scrollbar component for editor view
//!
//! Phase 3.1: Editor View Component Hierarchy

/// Scrollbar component
pub struct ScrollBar {
    /// Total height of content
    content_height: f32,
    /// Visible height (viewport)
    viewport_height: f32,
    /// Current scroll position
    scroll_position: f32,
    /// Width of scrollbar
    width: f32,
    /// Whether scrollbar is being dragged
    dragging: bool,
}

impl ScrollBar {
    pub fn new() -> Self {
        Self {
            content_height: 1000.0,
            viewport_height: 500.0,
            scroll_position: 0.0,
            width: 12.0,
            dragging: false,
        }
    }

    /// Set content and viewport heights
    pub fn set_dimensions(&mut self, content_height: f32, viewport_height: f32) {
        self.content_height = content_height.max(1.0);
        self.viewport_height = viewport_height.max(1.0);
    }

    /// Set current scroll position
    pub fn set_scroll_position(&mut self, position: f32) {
        let max_scroll = (self.content_height - self.viewport_height).max(0.0);
        self.scroll_position = position.max(0.0).min(max_scroll);
    }

    /// Get current scroll position
    pub fn scroll_position(&self) -> f32 {
        self.scroll_position
    }

    /// Calculate the height of the scrollbar thumb
    pub fn thumb_height(&self) -> f32 {
        let ratio = self.viewport_height / self.content_height;
        let min_thumb_height = 20.0;
        (self.viewport_height * ratio).max(min_thumb_height).min(self.viewport_height)
    }

    /// Calculate the Y position of the scrollbar thumb
    pub fn thumb_y(&self) -> f32 {
        let scrollable_height = self.content_height - self.viewport_height;
        if scrollable_height <= 0.0 {
            return 0.0;
        }

        let track_height = self.viewport_height - self.thumb_height();
        let thumb_position = (self.scroll_position / scrollable_height) * track_height;
        thumb_position
    }

    /// Handle click on scrollbar track
    pub fn on_track_click(&mut self, y: f32) {
        // Calculate scroll position based on click position
        let click_ratio = y / self.viewport_height;
        let scrollable_height = self.content_height - self.viewport_height;
        self.scroll_position = (click_ratio * scrollable_height).max(0.0);
    }

    /// Start dragging the thumb
    pub fn start_drag(&mut self) {
        self.dragging = true;
    }

    /// Stop dragging the thumb
    pub fn stop_drag(&mut self) {
        self.dragging = false;
    }

    /// Handle drag movement
    pub fn on_drag(&mut self, delta_y: f32, line_height: f32) {
        if !self.dragging {
            return;
        }

        let lines_to_scroll = delta_y / line_height;
        self.set_scroll_position(self.scroll_position + lines_to_scroll);
    }

    /// Check if scrollbar is needed
    pub fn is_needed(&self) -> bool {
        self.content_height > self.viewport_height
    }

    /// Check if thumb is at top
    pub fn is_at_top(&self) -> bool {
        self.scroll_position <= 0.0
    }

    /// Check if thumb is at bottom
    pub fn is_at_bottom(&self) -> bool {
        let scrollable_height = (self.content_height - self.viewport_height).max(0.0);
        self.scroll_position >= scrollable_height
    }

    /// Scroll up by one line
    pub fn scroll_up(&mut self, line_height: f32) {
        self.set_scroll_position(self.scroll_position - line_height);
    }

    /// Scroll down by one line
    pub fn scroll_down(&mut self, line_height: f32) {
        self.set_scroll_position(self.scroll_position + line_height);
    }

    /// Scroll by page
    pub fn scroll_page_up(&mut self) {
        self.set_scroll_position(self.scroll_position - self.viewport_height * 0.8);
    }

    /// Scroll by page down
    pub fn scroll_page_down(&mut self) {
        self.set_scroll_position(self.scroll_position + self.viewport_height * 0.8);
    }
}

impl Default for ScrollBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scrollbar_creation() {
        let scrollbar = ScrollBar::new();
        assert_eq!(scrollbar.scroll_position(), 0.0);
        assert!(scrollbar.is_at_top());
    }

    #[test]
    fn test_thumb_height() {
        let mut scrollbar = ScrollBar::new();
        scrollbar.set_dimensions(1000.0, 500.0);
        let thumb_height = scrollbar.thumb_height();
        assert!(thumb_height > 0.0);
        assert!(thumb_height <= 500.0);
    }

    #[test]
    fn test_scroll_position() {
        let mut scrollbar = ScrollBar::new();
        scrollbar.set_dimensions(1000.0, 500.0);
        scrollbar.set_scroll_position(100.0);
        assert_eq!(scrollbar.scroll_position(), 100.0);
    }

    #[test]
    fn test_scroll_boundaries() {
        let mut scrollbar = ScrollBar::new();
        scrollbar.set_dimensions(1000.0, 500.0);

        // Try to scroll beyond maximum
        scrollbar.set_scroll_position(10000.0);
        let max_scroll = 1000.0 - 500.0;
        assert_eq!(scrollbar.scroll_position(), max_scroll);
    }
}
