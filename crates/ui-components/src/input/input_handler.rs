//! Input handling for keyboard, mouse, and touch events
//!
//! Phase 3.4: Input Handling

use gpui::*;
use editor_core::Position;
use crate::input::key_bindings::{ Action, KeyBindings, Modifiers };
use std::time::{ Instant, Duration };

/// Input handler for the editor
pub struct InputHandler {
    /// Key bindings manager
    pub key_bindings: KeyBindings,
    /// IME composition state
    pub ime_state: ImeState,
    /// Last mouse click time and position for multi-click detection
    last_click: Option<(Instant, Point<Pixels>)>,
    /// Click count (1 = single, 2 = double, 3 = triple)
    click_count: u32,
    /// Mouse drag state
    drag_state: Option<DragState>,
}

/// State during mouse drag
#[derive(Clone, Debug)]
pub struct DragState {
    pub start_pos: Position,
    pub current_pos: Position,
    pub start_time: Instant,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            key_bindings: KeyBindings::load_defaults(),
            ime_state: ImeState::new(),
            last_click: None,
            click_count: 0,
            drag_state: None,
        }
    }

    /// Handle mouse click - classifies single/double/triple
    /// Returns the position where the click occurred (to be set as drag start)
    pub fn handle_mouse_down(&mut self, click_pos: Point<Pixels>) -> ClickClassification {
        let now = Instant::now();
        const DOUBLE_CLICK_TIMEOUT: Duration = Duration::from_millis(500);
        const DOUBLE_CLICK_DISTANCE: f32 = 5.0;

        let click_class = if let Some((last_time, last_pos)) = self.last_click {
            let time_delta = now.duration_since(last_time);
            // Convert Pixels to f32 for distance calculation
            let dx_pixels: f32 = (click_pos.x - last_pos.x).into();
            let dy_pixels: f32 = (click_pos.y - last_pos.y).into();
            let pos_delta = (dx_pixels * dx_pixels + dy_pixels * dy_pixels).sqrt();

            if time_delta < DOUBLE_CLICK_TIMEOUT && pos_delta < DOUBLE_CLICK_DISTANCE {
                self.click_count += 1;
                if self.click_count >= 3 {
                    ClickClassification::Triple
                } else {
                    ClickClassification::Double
                }
            } else {
                self.click_count = 1;
                ClickClassification::Single
            }
        } else {
            self.click_count = 1;
            ClickClassification::Single
        };

        self.last_click = Some((now, click_pos));

        // Reset drag state (will be initialized when drag starts)
        self.drag_state = None;

        click_class
    }

    /// Update drag state with new mouse position
    pub fn update_drag(&mut self, start_pos: Position, current_pos: Position) {
        if self.drag_state.is_none() {
            self.drag_state = Some(DragState {
                start_pos,
                current_pos,
                start_time: Instant::now(),
            });
        } else if let Some(ref mut drag) = self.drag_state {
            drag.current_pos = current_pos;
        }
    }

    /// End drag state
    pub fn end_drag(&mut self) -> Option<(Position, Position)> {
        self.drag_state.take().map(|drag| (drag.start_pos, drag.current_pos))
    }

    /// Get current drag state
    pub fn get_drag_state(&self) -> Option<&DragState> {
        self.drag_state.as_ref()
    }

    /// Handle keyboard input - convert key string to action
    pub fn handle_key(&self, key: &str, modifiers: Modifiers) -> Option<Action> {
        // Try to find a binding for this key combination
        self.key_bindings.find_action(key, modifiers)
    }

    /// Handle text input (from direct input, not special keys)
    pub fn handle_text_input(&mut self, text: &str) -> Option<Action> {
        // If text is printable and not from a special key, treat it as text insertion
        if !text.is_empty() && text.chars().all(|c| !c.is_control()) {
            return Some(Action::Insert(text.to_string()));
        }
        None
    }

    /// Handle mouse click
    pub fn handle_mouse_event(&mut self, _event: &MouseDownEvent) {
        // TODO: Implement mouse click handling for cursor positioning
    }

    /// Check if a key should produce text input based on its name
    pub fn is_text_input_key(key: &str) -> bool {
        // Single character keys (except special ones)
        if key.len() == 1 {
            let c = key.chars().next().unwrap();
            return !c.is_control() && !matches!(c, '\x1b' | '\x08' | '\x09');
        }

        // Special keys that produce text
        matches!(key, "Space")
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Classification of mouse click
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickClassification {
    Single,
    Double,
    Triple,
}

/// IME (Input Method Editor) state for CJK input
pub struct ImeState {
    pub composing: bool,
    pub composition: String,
    pub cursor_pos: usize,
}

impl ImeState {
    pub fn new() -> Self {
        Self {
            composing: false,
            composition: String::new(),
            cursor_pos: 0,
        }
    }

    pub fn start_composition(&mut self) {
        self.composing = true;
        self.composition.clear();
        self.cursor_pos = 0;
    }

    pub fn end_composition(&mut self) -> String {
        self.composing = false;
        let result = self.composition.clone();
        self.composition.clear();
        result
    }

    pub fn update_composition(&mut self, text: String) {
        self.composition = text;
    }
}

impl Default for ImeState {
    fn default() -> Self {
        Self::new()
    }
}

/// Mouse click type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickType {
    Single,
    Double,
    Triple,
    Quadruple,
}

/// Hover state management
pub struct HoverState {
    pub position: Point<Pixels>,
    pub start_time: std::time::Instant,
}
