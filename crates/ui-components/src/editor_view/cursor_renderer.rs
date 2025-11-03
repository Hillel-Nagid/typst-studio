//! Cursor rendering for primary and secondary cursors
//!
//! Phase 3.1: Editor View Component Hierarchy

use editor_core::{ Position, SelectionSet };
use gpui::{ point, px, size, Bounds, Hsla, Pixels, Point };
use std::time::{ Duration, Instant };

/// Cursor style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    /// Vertical line cursor (default)
    Line,
    /// Block cursor (covers character)
    Block,
    /// Underline cursor
    Underline,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Line
    }
}

/// Cursor blink state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlinkPhase {
    /// Cursor is visible
    Visible,
    /// Cursor is hidden
    Hidden,
}

/// Cursor renderer for multi-cursor support
pub struct CursorRenderer {
    /// Cursor style to use
    style: CursorStyle,
    /// Blink state for primary cursor
    blink_phase: BlinkPhase,
    /// Last blink time
    last_blink_time: Option<Instant>,
    /// Blink interval (on or off duration)
    blink_interval: Duration,
    /// Whether blinking is enabled
    blink_enabled: bool,
    /// Whether the cursor was recently moved (resets blink)
    cursor_moved: bool,
}

impl CursorRenderer {
    pub fn new() -> Self {
        Self {
            style: CursorStyle::default(),
            blink_phase: BlinkPhase::Visible,
            last_blink_time: None,
            blink_interval: Duration::from_millis(530), // Standard cursor blink rate
            blink_enabled: true,
            cursor_moved: false,
        }
    }

    /// Set the cursor style
    pub fn set_style(&mut self, style: CursorStyle) {
        self.style = style;
    }

    /// Get the current cursor style
    pub fn style(&self) -> CursorStyle {
        self.style
    }

    /// Enable or disable cursor blinking
    pub fn set_blink_enabled(&mut self, enabled: bool) {
        self.blink_enabled = enabled;
        if !enabled {
            self.blink_phase = BlinkPhase::Visible;
        }
    }

    /// Set the blink interval
    pub fn set_blink_interval(&mut self, interval: Duration) {
        self.blink_interval = interval;
    }

    /// Notify that the cursor has moved (resets blink to visible)
    pub fn on_cursor_moved(&mut self) {
        self.cursor_moved = true;
        self.blink_phase = BlinkPhase::Visible;
        self.last_blink_time = Some(Instant::now());
    }

    /// Update blink state
    pub fn update_blink(&mut self) {
        if !self.blink_enabled {
            return;
        }

        // If cursor just moved, reset blink timer
        if self.cursor_moved {
            self.cursor_moved = false;
            self.blink_phase = BlinkPhase::Visible;
            self.last_blink_time = Some(Instant::now());
            return;
        }

        let now = Instant::now();
        let last_blink = self.last_blink_time.unwrap_or(now);

        if now.duration_since(last_blink) >= self.blink_interval {
            self.blink_phase = match self.blink_phase {
                BlinkPhase::Visible => BlinkPhase::Hidden,
                BlinkPhase::Hidden => BlinkPhase::Visible,
            };
            self.last_blink_time = Some(now);
        }
    }

    /// Check if the primary cursor should be visible
    pub fn is_primary_visible(&self) -> bool {
        !self.blink_enabled || self.blink_phase == BlinkPhase::Visible
    }

    /// Render all cursors for a selection set
    pub fn render_cursors(
        &self,
        selections: &SelectionSet,
        line_height: f32,
        char_width: f32,
        viewport_offset: Point<Pixels>
    ) -> Vec<CursorShape> {
        let mut shapes = Vec::new();

        // Render all selections
        for (idx, selection) in selections.selections().iter().enumerate() {
            let is_primary = idx == 0; // Assuming primary is first

            // Only show primary cursor if blink phase allows it
            if is_primary && !self.is_primary_visible() {
                continue;
            }

            if
                let Some(shape) = self.render_cursor(
                    &selection.cursor.position,
                    line_height,
                    char_width,
                    viewport_offset,
                    is_primary
                )
            {
                shapes.push(shape);
            }
        }

        shapes
    }

    /// Render a single cursor at a position
    fn render_cursor(
        &self,
        position: &Position,
        line_height: f32,
        char_width: f32,
        viewport_offset: Point<Pixels>,
        is_primary: bool
    ) -> Option<CursorShape> {
        // Calculate cursor position in pixels
        let x = (position.column as f32) * char_width;
        let y = (position.line as f32) * line_height;

        // Apply viewport offset (convert to point for addition)
        let cursor_point = point(px(x), px(y));
        let screen_point = point(
            cursor_point.x + viewport_offset.x,
            cursor_point.y + viewport_offset.y
        );

        Some(CursorShape {
            bounds: self.cursor_bounds(screen_point, char_width, line_height),
            style: self.style,
            is_primary,
        })
    }

    /// Calculate cursor bounds based on style
    fn cursor_bounds(
        &self,
        origin: Point<Pixels>,
        char_width: f32,
        line_height: f32
    ) -> Bounds<Pixels> {
        match self.style {
            CursorStyle::Line => {
                // Thin vertical line
                let width = 2.0; // 2px wide line
                Bounds {
                    origin,
                    size: size(px(width), px(line_height)),
                }
            }
            CursorStyle::Block => {
                // Full character block
                Bounds {
                    origin,
                    size: size(px(char_width), px(line_height)),
                }
            }
            CursorStyle::Underline => {
                // Horizontal line at bottom
                let height = 2.0; // 2px tall line
                let underline_origin = point(origin.x, origin.y + px(line_height - height));
                Bounds {
                    origin: underline_origin,
                    size: size(px(char_width), px(height)),
                }
            }
        }
    }
}

impl Default for CursorRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Represents a cursor to be drawn
#[derive(Debug, Clone)]
pub struct CursorShape {
    /// Bounds of the cursor
    pub bounds: Bounds<Pixels>,
    /// Cursor style
    pub style: CursorStyle,
    /// Whether this is the primary cursor
    pub is_primary: bool,
}

impl CursorShape {
    /// Get the color for this cursor
    pub fn color(&self, primary_color: Hsla, secondary_color: Hsla) -> Hsla {
        if self.is_primary { primary_color } else { secondary_color }
    }
}

/// Primary cursor (blinking)
pub struct PrimaryCursor {
    position: Position,
    visible: bool,
}

impl PrimaryCursor {
    pub fn new(position: Position) -> Self {
        Self {
            position,
            visible: true,
        }
    }

    pub fn position(&self) -> Position {
        self.position
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }
}

/// Secondary cursors (multi-cursor mode)
pub struct SecondaryCursors {
    cursors: Vec<Position>,
}

impl SecondaryCursors {
    pub fn new() -> Self {
        Self {
            cursors: Vec::new(),
        }
    }

    pub fn add(&mut self, position: Position) {
        self.cursors.push(position);
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.cursors.len() {
            self.cursors.remove(index);
        }
    }

    pub fn clear(&mut self) {
        self.cursors.clear();
    }

    pub fn positions(&self) -> &[Position] {
        &self.cursors
    }

    pub fn count(&self) -> usize {
        self.cursors.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cursors.is_empty()
    }
}

impl Default for SecondaryCursors {
    fn default() -> Self {
        Self::new()
    }
}
