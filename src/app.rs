//! Main application structure and GPUI integration

use crate::state::{ ApplicationState, WindowState };
use editor_core::{ Buffer, BufferId, Position, Version };
use gpui::*;
use ui_components::{ EditorView, editor_view::TopNav };
use ui_components::input::{ InputHandler };
use ui_components::input::key_bindings::Action;
use ui_components::syntax::highlighting::{ SyntaxHighlighter, HighlightResult };
use ui_components::rendering::{ TextShaper, FontManager, FontData, BidiShapedText };
use bidi_text::Direction;
use std::path::PathBuf;
use std::sync::Arc;
use std::collections::HashMap;

/// The main Typst Studio application state manager
pub struct TypstEditor {
    /// Application state
    pub state: ApplicationState,
    /// Buffer registry
    buffers: std::collections::HashMap<BufferId, Buffer>,
    /// Next buffer ID
    next_buffer_id: u64,
}

impl TypstEditor {
    pub fn new() -> Self {
        Self {
            state: ApplicationState::new(),
            buffers: std::collections::HashMap::new(),
            next_buffer_id: 1,
        }
    }

    /// Create a new buffer
    pub fn create_buffer(&mut self, text: &str) -> BufferId {
        let id = BufferId::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        let buffer = Buffer::from_text(id, text);
        self.buffers.insert(id, buffer);

        id
    }

    /// Get a buffer by ID
    pub fn get_buffer(&self, id: BufferId) -> Option<&Buffer> {
        self.buffers.get(&id)
    }

    /// Get a mutable buffer by ID
    pub fn get_buffer_mut(&mut self, id: BufferId) -> Option<&mut Buffer> {
        self.buffers.get_mut(&id)
    }

    /// Open a file
    pub fn open_file(&mut self, path: PathBuf) -> Result<BufferId, std::io::Error> {
        let id = BufferId::new(self.next_buffer_id);
        self.next_buffer_id += 1;

        let buffer = Buffer::from_file(id, path.clone())?;
        self.buffers.insert(id, buffer);

        self.state.add_recent_file(path);

        Ok(id)
    }

    /// Create a new window
    pub fn new_window(&mut self) -> usize {
        let window_id = self.state.windows.len();
        let window = WindowState::new(window_id);
        self.state.add_window(window);
        window_id
    }

    /// Get application state
    pub fn state(&self) -> &ApplicationState {
        &self.state
    }

    /// Get mutable application state
    pub fn state_mut(&mut self) -> &mut ApplicationState {
        &mut self.state
    }
}

impl Default for TypstEditor {
    fn default() -> Self {
        Self::new()
    }
}

/// GPUI Window component for the editor
pub struct TypstEditorWindow {
    app: TypstEditor,
    editor: EditorView,
    top_nav: TopNav,
    input_handler: InputHandler,
    active_buffer_id: Option<BufferId>,
    syntax_highlighter: SyntaxHighlighter,
    /// Cache: (buffer_id, buffer_version) -> HighlightResult
    highlight_cache: HashMap<(BufferId, Version), Arc<HighlightResult>>,
    /// Text shaper for complex script support
    text_shaper: TextShaper,
    /// Font manager for font loading
    font_manager: FontManager,
    /// Default font for rendering
    default_font: Option<Arc<FontData>>,
}

impl TypstEditorWindow {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        // Create application state
        let mut app = TypstEditor::new();

        // Create a sample buffer with Typst content
        let sample_content =
            r#"
+ 
 נפריך את הטענה.\
 מתקיים מסכום גיאומטרי
 $
n sum_(i=1)^(n^2) 1/2^i = 2n (1-1/2^(n^2)) = Theta(n)!=Theta(n^2)
$
 ולכן הטענה אינה נכונה
+ 
 נוכיח את הטענה\
 מהגדרת בינום מתקיים
 $
binom(n,2)=1/2(n!)/(n-2)! =n/2(n-1)=Theta(n^2)
$
+
 נוכיח את הטענה\
 נשים לב כי מתקיים
 $
ln(n!)=sum_(i=1)^n ln(i)\
integral_1^n ln(x)d\x=[x ln(x)-x]_1^n=n ln(n)-n+1=Omega(n ln n)
$
 מתקיים 
 $
integral_1^n ln(x)d\x = Theta(sum_(i=1)^n ln(i))
$
 ומכאן 
 $
ln(n!)=sum_(i=1)^n ln(i)=Omega(n ln n)
$ 
+ 
 נפריך את הטענה
 $
log sqrt(n)=log n^(1/2)=1/2 log n=Theta(log n)
$
 מכלל ההצבה ומשפט לופיטל נשים לב כי מתקיים
 $
lim_(n->infinity)sqrt(log (n))/log(n)=lim_(x->infinity)sqrt(x)/x=lim_(x->infinity)1/(2x)=0!=infinity
$
 כלומר נקבל מההגדרה כי $sqrt(log n)!=omega(log n)$ ולכן בפרט $sqrt(log n)!=Theta(log n)=Theta(log sqrt(n))$
+ 
 נפריך את הטענה ע"י דוגמה נגדית\
 נגדיר $f(n)=cases(n^2 quad "even",1 quad "odd"),g(n)=cases(1 quad "even",n^2 quad "odd")$\
 נשים לב כי לכל מספר זוגי מתקיים $f(n)=Omega(g(n))$ וכן $g(n)=Omega(f(n))$ לכל מספר אי זוגי, אך בנוסף לא מתקיים $1=Omega(n^2)$ ולכן שתי הטענות אינן מתקיימות בסתירה לטענה
+ 
 נוכיח את הטענה\
 נניח כי $f(n)!=Omega(g(n))$, כלומר מתקיים לכל $c in RR^+$
 $
f(n)<c dot g(n)<=>1/c f(n)< g(n)
$
 ולכן מההגדרה מתקיים $g(n)=Omega(f(n))$ ממונוטוניות הפונקציות"#;

        let buffer_id = app.create_buffer(sample_content);

        // Create the editor view
        let mut editor = EditorView::new();
        editor.set_buffer(buffer_id);

        // Create top nav
        let top_nav = TopNav::new();

        // Create input handler
        let input_handler = InputHandler::new();

        // Create font manager and load default font
        let mut font_manager = FontManager::new();
        let default_font = font_manager
            .load_font("Courier New", 400, false)
            .or_else(|| font_manager.load_font("monospace", 400, false))
            .or_else(|| font_manager.load_font("Arial", 400, false));

        Self {
            app,
            editor,
            top_nav,
            input_handler,
            active_buffer_id: Some(buffer_id),
            syntax_highlighter: SyntaxHighlighter::new(),
            highlight_cache: HashMap::new(),
            text_shaper: TextShaper::new(),
            font_manager,
            default_font,
        }
    }

    /// Execute an editor action on the buffer
    fn execute_action(&mut self, action: Action) {
        let buffer_id = match self.active_buffer_id {
            Some(id) => id,
            None => {
                return;
            }
        };

        let buffer = match self.app.get_buffer_mut(buffer_id) {
            Some(buf) => buf,
            None => {
                return;
            }
        };

        let mut cursor_pos = self.editor.get_cursor_position();

        match action {
            Action::Insert(text) => {
                if let Ok(()) = buffer.insert(cursor_pos, &text) {
                    // Move cursor after inserted text
                    let lines_added = text.matches('\n').count();
                    if lines_added > 0 {
                        let last_line_len = text.lines().last().unwrap_or("").len();
                        cursor_pos = Position::new(cursor_pos.line + lines_added, last_line_len);
                    } else {
                        cursor_pos = Position::new(cursor_pos.line, cursor_pos.column + text.len());
                    }
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::Backspace => {
                if let Ok(new_pos) = buffer.backspace(cursor_pos) {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::Delete => {
                if let Ok(new_pos) = buffer.delete_forward(cursor_pos) {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::Newline => {
                if let Ok(()) = buffer.insert(cursor_pos, "\n") {
                    cursor_pos = Position::new(cursor_pos.line + 1, 0);
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::MoveLeft => {
                if cursor_pos.column > 0 {
                    cursor_pos.column -= 1;
                } else if cursor_pos.line > 0 {
                    cursor_pos.line -= 1;
                    if let Ok(line_text) = buffer.line(cursor_pos.line) {
                        cursor_pos.column = line_text.len();
                    }
                }
                self.editor.set_cursor_position(cursor_pos);
            }

            Action::MoveRight => {
                if let Ok(line_text) = buffer.line(cursor_pos.line) {
                    if cursor_pos.column < line_text.len() {
                        cursor_pos.column += 1;
                    } else if cursor_pos.line + 1 < buffer.len_lines() {
                        cursor_pos.line += 1;
                        cursor_pos.column = 0;
                    }
                }
                self.editor.set_cursor_position(cursor_pos);
            }

            Action::MoveUp => {
                if cursor_pos.line > 0 {
                    cursor_pos.line -= 1;
                    if let Ok(line_text) = buffer.line(cursor_pos.line) {
                        cursor_pos.column = cursor_pos.column.min(line_text.len());
                    }
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::MoveDown => {
                if cursor_pos.line + 1 < buffer.len_lines() {
                    cursor_pos.line += 1;
                    if let Ok(line_text) = buffer.line(cursor_pos.line) {
                        cursor_pos.column = cursor_pos.column.min(line_text.len());
                    }
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::MoveLineStart => {
                cursor_pos.column = 0;
                self.editor.set_cursor_position(cursor_pos);
            }

            Action::MoveLineEnd => {
                if let Ok(line_text) = buffer.line(cursor_pos.line) {
                    cursor_pos.column = line_text.len();
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::MoveDocumentStart => {
                self.editor.set_cursor_position(Position::new(0, 0));
            }

            Action::MoveDocumentEnd => {
                let last_line = buffer.len_lines().saturating_sub(1);
                if let Ok(line_text) = buffer.line(last_line) {
                    self.editor.set_cursor_position(Position::new(last_line, line_text.len()));
                }
            }

            Action::Undo => {
                if let Ok(new_pos) = buffer.undo() {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::Redo => {
                if let Ok(new_pos) = buffer.redo() {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::MoveWordLeft => {
                if let Ok(new_pos) = buffer.prev_word_boundary(cursor_pos) {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::MoveWordRight => {
                if let Ok(new_pos) = buffer.next_word_boundary(cursor_pos) {
                    self.editor.set_cursor_position(new_pos);
                }
            }

            Action::Indent => {
                if let Ok(()) = buffer.insert(cursor_pos, "    ") {
                    cursor_pos = Position::new(cursor_pos.line, cursor_pos.column + 4);
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            Action::DeleteWord => {
                if let Ok(end) = buffer.next_word_boundary(cursor_pos) {
                    let _ = buffer.delete(cursor_pos, end);
                    self.editor.set_cursor_position(cursor_pos);
                }
            }

            _ => {
                // TODO: Implement remaining actions (selection, clipboard, etc.)
            }
        }
    }

    /// Handle keyboard input
    pub fn on_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>
    ) -> bool {
        use ui_components::input::key_bindings::Modifiers;

        // Convert GPUI keystroke to our key string format
        let key_str = event.keystroke.key.as_str();

        // Build modifiers
        let modifiers = Modifiers {
            ctrl: event.keystroke.modifiers.control,
            alt: event.keystroke.modifiers.alt,
            shift: event.keystroke.modifiers.shift,
            meta: event.keystroke.modifiers.platform,
        };

        // Try to get an action from the input handler
        if let Some(action) = self.input_handler.handle_key(key_str, modifiers) {
            self.execute_action(action);
            cx.notify();
            return true;
        }

        // If no binding found, check if it's a text input
        if let Some(action) = self.input_handler.handle_text_input(key_str) {
            self.execute_action(action);
            cx.notify();
            return true;
        }

        false
    }

    /// Get buffer content lines for rendering
    fn get_buffer_lines(&self, max_lines: usize) -> Vec<String> {
        if let Some(buffer_id) = self.editor.buffer_id() {
            if let Some(buffer) = self.app.get_buffer(buffer_id) {
                let line_count = buffer.len_lines().min(max_lines);
                (0..line_count).filter_map(|i| buffer.line(i).ok()).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    /// Get highlighted tokens for the active buffer (with caching)
    fn get_highlights(&mut self) -> Option<Arc<HighlightResult>> {
        let buffer_id = self.active_buffer_id?;
        let buffer = self.app.get_buffer(buffer_id)?;
        let version = buffer.version();

        // Check cache
        let cache_key = (buffer_id, version);
        if let Some(cached) = self.highlight_cache.get(&cache_key) {
            return Some(cached.clone());
        }

        // Highlight and cache
        let text = buffer.text();
        let highlights = self.syntax_highlighter.highlight(&text);
        self.highlight_cache.insert(cache_key, highlights.clone());
        Some(highlights)
    }

    /// Render a line with bidirectional text support
    /// Returns styled text with proper RTL/LTR handling
    fn render_bidi_line(
        &mut self,
        line_text: &str,
        highlights: Option<&Arc<HighlightResult>>
    ) -> AnyElement {
        // Shape the text with bidi support
        let shaped_text = if let Some(ref font) = self.default_font {
            self.text_shaper.shape_with_bidi(line_text, font)
        } else {
            // Fallback: create a simple bidi-aware text without shaping
            use bidi_text::BidiParagraph;
            let para = BidiParagraph::new(line_text.to_string(), None);

            // Create a simple shaped text structure
            BidiShapedText {
                base_direction: para.base_direction(),
                runs: para
                    .visual_runs()
                    .into_iter()
                    .map(|run| {
                        use ui_components::rendering::{ BidiShapedRun, ShapedText, ShapedGlyph };
                        let run_text = &line_text[run.logical_range.clone()];
                        BidiShapedRun {
                            logical_range: run.logical_range,
                            direction: run.direction,
                            shaped_text: ShapedText {
                                glyphs: run_text
                                    .chars()
                                    .enumerate()
                                    .map(|(i, ch)| ShapedGlyph {
                                        glyph_id: ch as u32,
                                        cluster: i as u32,
                                        x_offset: 0.0,
                                        y_offset: 0.0,
                                        x_advance: 8.0,
                                        y_advance: 0.0,
                                    })
                                    .collect(),
                            },
                            level: run.level,
                        }
                    })
                    .collect(),
                full_text: line_text.to_string(),
            }
        };

        // Create spans for each bidi run
        let mut spans: Vec<AnyElement> = Vec::new();

        for (_run_idx, run) in shaped_text.runs.iter().enumerate() {
            let run_text = &line_text[run.logical_range.clone()];

            // For RTL runs, we may need to reverse the visual display
            let display_text = if run.direction == Direction::RightToLeft {
                // Reverse the string for RTL text
                run_text.chars().rev().collect::<String>()
            } else {
                run_text.to_string()
            };

            // Get color from syntax highlighting if available
            let color = if let Some(hl) = highlights {
                // Find token that overlaps with this run
                hl.tokens
                    .iter()
                    .find(|t| t.start < run.logical_range.end && t.end > run.logical_range.start)
                    .map(|t| t.color)
                    .unwrap_or(rgb(0xcccccc))
            } else {
                rgb(0xcccccc)
            };

            spans.push(
                div().child(display_text).text_color(color).text_size(px(13.0)).into_any_element()
            );
        }

        // For RTL base direction, reverse the order of spans
        if shaped_text.base_direction == Direction::RightToLeft {
            spans.reverse();
        }

        div().flex().children(spans).into_any_element()
    }

    /// Build a styled line element from tokens
    /// Returns a div with colored text runs based on tokens
    fn build_styled_line(&self, line_text: &str, tokens: &[Arc<HighlightResult>]) -> AnyElement {
        let mut spans: Vec<AnyElement> = Vec::new();
        let line_bytes = line_text.as_bytes();
        let mut last_end = 0;

        // If we have tokens, render with colors
        if let Some(highlights) = tokens.first() {
            for token in &highlights.tokens {
                // Add plain text before this token
                if last_end < token.start && token.start < line_bytes.len() {
                    if let Ok(text) = std::str::from_utf8(&line_bytes[last_end..token.start]) {
                        spans.push(
                            div()
                                .child(text.to_string())
                                .text_color(rgb(0xcccccc))
                                .text_size(px(13.0))
                                .into_any_element()
                        );
                    }
                }

                // Add colored token
                if token.end <= line_bytes.len() {
                    if let Ok(text) = std::str::from_utf8(&line_bytes[token.start..token.end]) {
                        spans.push(
                            div()
                                .child(text.to_string())
                                .text_color(token.color)
                                .text_size(px(13.0))
                                .into_any_element()
                        );
                    }
                }

                last_end = token.end;
            }

            // Add remaining text after last token
            if last_end < line_bytes.len() {
                if let Ok(text) = std::str::from_utf8(&line_bytes[last_end..]) {
                    spans.push(
                        div()
                            .child(text.to_string())
                            .text_color(rgb(0xcccccc))
                            .text_size(px(13.0))
                            .into_any_element()
                    );
                }
            }
        } else {
            // No highlighting, render as plain text
            spans.push(
                div()
                    .child(line_text.to_string())
                    .text_color(rgb(0xcccccc))
                    .text_size(px(13.0))
                    .into_any_element()
            );
        }

        div().children(spans).into_any_element()
    }
}

impl Render for TypstEditorWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0x1e1e1e))
            .on_key_down(
                _cx.listener(|this, event: &KeyDownEvent, window: &mut Window, cx| {
                    this.on_key_down(event, window, cx);
                })
            )
            // Custom Title Bar
            .child(
                div()
                    .w_full()
                    .h(px(36.0))
                    .bg(rgb(0x2d2d30))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(12.0))
                    .on_mouse_down(
                        MouseButton::Left,
                        _cx.listener(|_this, _event: &MouseDownEvent, window: &mut Window, _cx| {
                            window.start_window_move();
                        })
                    )
                    // Left section: Logo + Title
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .min_w(px(200.0))
                            // Logo
                            .child(div().child("▶").text_color(rgb(0x007acc)).text_size(px(16.0)))
                            // Title text
                            .child(
                                div()
                                    .child("Typst Studio")
                                    .text_color(rgb(0xcccccc))
                                    .text_size(px(14.0))
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                            )
                    )
                    // Middle section: Menu Bar
                    .child(
                        div()
                            .flex()
                            .gap(px(0.0))
                            .flex_1()
                            .justify_center()
                            .children(
                                self.top_nav.menu_bar.menus.iter().map(|menu| {
                                    div()
                                        .px(px(12.0))
                                        .py(px(8.0))
                                        .child(menu.title.clone())
                                        .text_color(rgb(0xcccccc))
                                        .text_size(px(13.0))
                                        .hover(|style| style.bg(rgb(0x3e3e42)))
                                        .on_mouse_down(
                                            MouseButton::Left,
                                            _cx.listener(
                                                |
                                                    _this,
                                                    _event: &MouseDownEvent,
                                                    _window: &mut Window,
                                                    _cx
                                                | {
                                                    // Prevent window dragging when clicking menu items
                                                }
                                            )
                                        )
                                })
                            )
                    )
                    // Right section: Window Controls
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(0.0))
                            .min_w(px(138.0))
                            // Minimize button
                            .child(
                                div()
                                    .child("−")
                                    .text_color(rgb(0xcccccc))
                                    .text_size(px(18.0))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .hover(|style| style.bg(rgb(0x3e3e42)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        _cx.listener(
                                            |
                                                _this,
                                                _event: &MouseDownEvent,
                                                window: &mut Window,
                                                _cx
                                            | {
                                                window.minimize_window();
                                            }
                                        )
                                    )
                            )
                            // Maximize button
                            .child(
                                div()
                                    .child("□")
                                    .text_color(rgb(0xcccccc))
                                    .text_size(px(14.0))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .hover(|style| style.bg(rgb(0x3e3e42)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        _cx.listener(
                                            |
                                                _this,
                                                _event: &MouseDownEvent,
                                                window: &mut Window,
                                                _cx
                                            | {
                                                window.toggle_fullscreen();
                                            }
                                        )
                                    )
                            )
                            // Close button
                            .child(
                                div()
                                    .child("✕")
                                    .text_color(rgb(0xffffff))
                                    .text_size(px(14.0))
                                    .bg(rgb(0xc42e1e))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .hover(|style| style.bg(rgb(0xe81123)))
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        _cx.listener(
                                            |
                                                _this,
                                                _event: &MouseDownEvent,
                                                _window: &mut Window,
                                                _cx
                                            | {
                                                // For now, we'll just print a message. Full window close would require different approach
                                                // The window typically closes when the last entity is removed
                                                tracing::info!("Close button clicked");
                                            }
                                        )
                                    )
                            )
                    )
            )
            // Split Pane Layout: Editor | Preview
            .child(
                div()
                    .flex_1()
                    .flex()
                    .overflow_hidden()
                    .bg(rgb(0x1e1e1e))
                    // Left pane: EDITOR
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .bg(rgb(0x1e1e1e))
                            // Editor label
                            .child(
                                div()
                                    .w_full()
                                    .h(px(32.0))
                                    .bg(rgb(0x2d2d30))
                                    .flex()
                                    .items_center()
                                    .px(px(12.0))
                                    .child("EDITOR")
                                    .text_color(rgb(0xcccccc))
                                    .text_size(px(12.0))
                            )
                            // Editor content
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .overflow_hidden()
                                    // Gutter
                                    .child(
                                        div()
                                            .w(px(self.editor.gutter.calculate_width(100)))
                                            .h_full()
                                            .bg(rgb(0x252526))
                                            .flex()
                                            .flex_col()
                                            .overflow_hidden()
                                            .px(px(4.0))
                                            .py(px(8.0))
                                            .children(
                                                (0..20).map(|line| {
                                                    div()
                                                        .h(px(self.editor.text_content.line_height))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .child(
                                                            div()
                                                                .child(format!("{}", line + 1))
                                                                .text_color(rgb(0x858585))
                                                                .text_size(px(12.0))
                                                        )
                                                })
                                            )
                                    )
                                    // Text content area with syntax highlighting
                                    .child(
                                        div()
                                            .flex_1()
                                            .flex_col()
                                            .px(px(8.0))
                                            .py(px(8.0))
                                            .on_mouse_down(
                                                MouseButton::Left,
                                                _cx.listener(
                                                    |
                                                        this,
                                                        event: &MouseDownEvent,
                                                        _window: &mut Window,
                                                        cx
                                                    | {
                                                        let mouse_pos = event.position;
                                                        let gutter_width =
                                                            this.editor.gutter.calculate_width(100);
                                                        let char_width =
                                                            this.editor.text_content.char_width;
                                                        let line_height =
                                                            this.editor.text_content.line_height;

                                                        // Calculate position relative to text area (accounting for padding)
                                                        let content_x: f32 = (
                                                            mouse_pos.x -
                                                            px(gutter_width) -
                                                            px(8.0)
                                                        ).into();
                                                        let content_y: f32 = (
                                                            mouse_pos.y - px(8.0)
                                                        ).into();

                                                        let position =
                                                            EditorView::point_to_position(
                                                                content_x,
                                                                content_y,
                                                                char_width,
                                                                line_height
                                                            );

                                                        // Clamp position to valid buffer range
                                                        if
                                                            let Some(buffer_id) =
                                                                this.active_buffer_id
                                                        {
                                                            if
                                                                let Some(buffer) =
                                                                    this.app.get_buffer(buffer_id)
                                                            {
                                                                let max_line = buffer
                                                                    .len_lines()
                                                                    .saturating_sub(1);
                                                                let clamped_line =
                                                                    position.line.min(max_line);

                                                                let clamped_col = if
                                                                    let Ok(line_text) =
                                                                        buffer.line(clamped_line)
                                                                {
                                                                    position.column.min(
                                                                        line_text.len()
                                                                    )
                                                                } else {
                                                                    0
                                                                };

                                                                let clamped_position =
                                                                    Position::new(
                                                                        clamped_line,
                                                                        clamped_col
                                                                    );
                                                                this.editor.set_cursor_position(
                                                                    clamped_position
                                                                );

                                                                // Handle click and initialize drag
                                                                let _ =
                                                                    this.input_handler.handle_mouse_down(
                                                                        mouse_pos
                                                                    );
                                                                // Initialize drag state with the clicked position
                                                                this.input_handler.update_drag(
                                                                    clamped_position,
                                                                    clamped_position
                                                                );
                                                            }
                                                        }
                                                        cx.notify();
                                                    }
                                                )
                                            )
                                            .on_mouse_move(
                                                _cx.listener(
                                                    |
                                                        this,
                                                        event: &MouseMoveEvent,
                                                        _window: &mut Window,
                                                        cx
                                                    | {
                                                        let mouse_pos = event.position;

                                                        // Only update if we're dragging (mouse is down)
                                                        if
                                                            this.input_handler
                                                                .get_drag_state()
                                                                .is_some()
                                                        {
                                                            let gutter_width =
                                                                this.editor.gutter.calculate_width(
                                                                    100
                                                                );
                                                            let char_width =
                                                                this.editor.text_content.char_width;
                                                            let line_height =
                                                                this.editor.text_content.line_height;

                                                            let content_x: f32 = (
                                                                mouse_pos.x -
                                                                px(gutter_width) -
                                                                px(8.0)
                                                            ).into();
                                                            let content_y: f32 = (
                                                                mouse_pos.y - px(8.0)
                                                            ).into();

                                                            let position =
                                                                EditorView::point_to_position(
                                                                    content_x,
                                                                    content_y,
                                                                    char_width,
                                                                    line_height
                                                                );

                                                            // Update drag state and selection
                                                            let start_pos = this.input_handler
                                                                .get_drag_state()
                                                                .map(|d| d.start_pos)
                                                                .unwrap_or(
                                                                    this.editor.get_cursor_position()
                                                                );

                                                            this.input_handler.update_drag(
                                                                start_pos,
                                                                position
                                                            );

                                                            // Update cursor to current position
                                                            if
                                                                let Some(buffer_id) =
                                                                    this.active_buffer_id
                                                            {
                                                                if
                                                                    let Some(buffer) =
                                                                        this.app.get_buffer(
                                                                            buffer_id
                                                                        )
                                                                {
                                                                    let max_line = buffer
                                                                        .len_lines()
                                                                        .saturating_sub(1);
                                                                    let clamped_line =
                                                                        position.line.min(max_line);

                                                                    let clamped_col = if
                                                                        let Ok(line_text) =
                                                                            buffer.line(
                                                                                clamped_line
                                                                            )
                                                                    {
                                                                        position.column.min(
                                                                            line_text.len()
                                                                        )
                                                                    } else {
                                                                        0
                                                                    };

                                                                    let clamped_position =
                                                                        Position::new(
                                                                            clamped_line,
                                                                            clamped_col
                                                                        );
                                                                    this.editor.set_cursor_position(
                                                                        clamped_position
                                                                    );
                                                                }
                                                            }

                                                            cx.notify();
                                                        }
                                                    }
                                                )
                                            )
                                            .on_mouse_up(
                                                MouseButton::Left,
                                                _cx.listener(
                                                    |
                                                        this,
                                                        _event: &MouseUpEvent,
                                                        _window: &mut Window,
                                                        cx
                                                    | {
                                                        this.input_handler.end_drag();
                                                        cx.notify();
                                                    }
                                                )
                                            )
                                            .children({
                                                let lines = self.get_buffer_lines(20);
                                                let _highlights = self.get_highlights();
                                                let cursor_line =
                                                    self.editor.get_cursor_position().line;
                                                let cursor_col =
                                                    self.editor.get_cursor_position().column;
                                                let line_height =
                                                    self.editor.text_content.line_height;
                                                let char_width =
                                                    self.editor.text_content.char_width;
                                                let is_primary_visible =
                                                    self.editor.cursor_renderer.is_primary_visible();

                                                // Compute bidi layout for all lines upfront
                                                let bidi_layouts: Vec<BidiShapedText> = lines
                                                    .iter()
                                                    .map(|line| {
                                                        let line_text = line.trim_end_matches('\n');

                                                        // Create bidi layout without shaping (fallback mode)
                                                        use bidi_text::BidiParagraph;
                                                        let para = BidiParagraph::new(
                                                            line_text.to_string(),
                                                            None
                                                        );

                                                        BidiShapedText {
                                                            base_direction: para.base_direction(),
                                                            runs: para
                                                                .visual_runs()
                                                                .into_iter()
                                                                .map(|run| {
                                                                    use ui_components::rendering::{
                                                                        BidiShapedRun,
                                                                        ShapedText,
                                                                        ShapedGlyph,
                                                                    };
                                                                    let run_text =
                                                                        &line_text
                                                                            [
                                                                                run.logical_range.clone()
                                                                            ];
                                                                    BidiShapedRun {
                                                                        logical_range: run.logical_range,
                                                                        direction: run.direction,
                                                                        shaped_text: ShapedText {
                                                                            glyphs: run_text
                                                                                .chars()
                                                                                .enumerate()
                                                                                .map(
                                                                                    |(
                                                                                        i,
                                                                                        ch,
                                                                                    )| ShapedGlyph {
                                                                                        glyph_id: ch as u32,
                                                                                        cluster: i as u32,
                                                                                        x_offset: 0.0,
                                                                                        y_offset: 0.0,
                                                                                        x_advance: 8.0,
                                                                                        y_advance: 0.0,
                                                                                    }
                                                                                )
                                                                                .collect(),
                                                                        },
                                                                        level: run.level,
                                                                    }
                                                                })
                                                                .collect(),
                                                            full_text: line_text.to_string(),
                                                        }
                                                    })
                                                    .collect();

                                                lines
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(move |(i, line)| {
                                                        let is_cursor_line = i == cursor_line;
                                                        let line_text = line
                                                            .trim_end_matches('\n')
                                                            .to_string();

                                                        // Get the bidi layout for this line
                                                        let bidi_layout = &bidi_layouts[i];

                                                        // Create spans for each bidi run
                                                        let mut spans: Vec<AnyElement> = Vec::new();

                                                        for run in &bidi_layout.runs {
                                                            let run_text =
                                                                &line_text
                                                                    [run.logical_range.clone()];

                                                            // For RTL runs, reverse the visual display
                                                            let display_text = if
                                                                run.direction ==
                                                                Direction::RightToLeft
                                                            {
                                                                run_text
                                                                    .chars()
                                                                    .rev()
                                                                    .collect::<String>()
                                                            } else {
                                                                run_text.to_string()
                                                            };

                                                            spans.push(
                                                                div()
                                                                    .child(display_text)
                                                                    .text_color(rgb(0xcccccc))
                                                                    .text_size(px(13.0))
                                                                    .into_any_element()
                                                            );
                                                        }

                                                        // For RTL base direction, reverse the order of spans
                                                        if
                                                            bidi_layout.base_direction ==
                                                            Direction::RightToLeft
                                                        {
                                                            spans.reverse();
                                                        }

                                                        let line_content = div()
                                                            .flex()
                                                            .children(spans);

                                                        let mut line_div = div()
                                                            .h(px(line_height))
                                                            .flex()
                                                            .items_center()
                                                            .child(line_content);

                                                        // Add cursor if this is the cursor line
                                                        if is_cursor_line && is_primary_visible {
                                                            let cursor_x =
                                                                (cursor_col as f32) * char_width;
                                                            line_div = line_div.child(
                                                                div()
                                                                    .absolute()
                                                                    .left(px(cursor_x))
                                                                    .top(px(0.0))
                                                                    .w(px(2.0))
                                                                    .h(px(line_height))
                                                                    .bg(rgb(0x007acc))
                                                            );
                                                        }

                                                        line_div.into_any_element()
                                                    })
                                                    .collect::<Vec<_>>()
                                            })
                                    )
                                    // Scrollbar
                                    .child(
                                        div()
                                            .w(px(12.0))
                                            .h_full()
                                            .bg(rgb(0x1e1e1e))
                                            .flex()
                                            .justify_center()
                                            .py(px(2.0))
                                            .child(
                                                div()
                                                    .w(px(8.0))
                                                    .h(px(60.0))
                                                    .rounded(px(4.0))
                                                    .bg(rgb(0x464647))
                                            )
                                    )
                            )
                    )
                    // Divider
                    .child(div().w(px(1.0)).h_full().bg(rgb(0x3e3e42)))
                    // Right pane: PREVIEW
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .bg(rgb(0x2d2d30))
                            // Preview label
                            .child(
                                div()
                                    .w_full()
                                    .h(px(32.0))
                                    .bg(rgb(0x2d2d30))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px(px(12.0))
                                    .child(
                                        div()
                                            .child("PREVIEW")
                                            .text_color(rgb(0xcccccc))
                                            .text_size(px(12.0))
                                    )
                                    .child(
                                        div()
                                            .child("Uprarent")
                                            .text_color(rgb(0x858585))
                                            .text_size(px(11.0))
                                    )
                                    .child(
                                        div()
                                            .child("Roptuile Ple Ln3")
                                            .text_color(rgb(0x858585))
                                            .text_size(px(11.0))
                                    )
                            )
                            // Preview content area
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .overflow_hidden()
                                    .bg(rgb(0x2d2d30))
                                    .px(px(16.0))
                                    .py(px(16.0))
                                    // White document area
                                    .child(
                                        div()
                                            .w(px(400.0))
                                            .h(px(600.0))
                                            .bg(rgb(0xffffff))
                                            .rounded(px(2.0))
                                            .flex()
                                            .flex_col()
                                            .px(px(40.0))
                                            .py(px(40.0))
                                            .gap(px(12.0))
                                            // Document heading
                                            .child(
                                                div()
                                                    .child("Theorem")
                                                    .text_color(rgb(0x000000))
                                                    .text_size(px(24.0))
                                            )
                                            // Document text
                                            .child(
                                                div()
                                                    .child(
                                                        "To reorpois intsistent veil enxom quseit-math leg tisifie tihe momoeott con content n stum amore neque, sed thes timelyeu ais avxocte arceex set enoew s LIIB. ske sis tedui. Co 1t D 15 D; suibt, ts Biessce Sieet jegis ts nchos ppe kolderpe."
                                                    )
                                                    .text_color(rgb(0x333333))
                                                    .text_size(px(14.0))
                                            )
                                            // Document math
                                            .child(
                                                div()
                                                    .child("$ ∫0¹ = x²/2  dx $")
                                                    .text_color(rgb(0x000000))
                                                    .text_size(px(16.0))
                                            )
                                            // More document text
                                            .child(
                                                div()
                                                    .child(
                                                        "We oisons ing: trAts, Ixselle thera eh s entieleing aad be pasotte vie es lves ev Bnee hei ho I His wis eni hshcit heme Bascul aas bavygire tnousst anda tueak its ex, itlaced Colorcilied."
                                                    )
                                                    .text_color(rgb(0x333333))
                                                    .text_size(px(14.0))
                                            )
                                            // Theorem heading
                                            .child(
                                                div()
                                                    .child("Theorem")
                                                    .text_color(rgb(0x000000))
                                                    .text_size(px(16.0))
                                            )
                                    )
                            )
                    )
            )
            // Status Bar at bottom
            .child(
                div()
                    .w_full()
                    .h(px(24.0))
                    .bg(rgb(0x007acc))
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(12.0))
                    .child({
                        let cursor_pos = self.editor.get_cursor_position();
                        let dirty_indicator = if let Some(buffer_id) = self.editor.buffer_id() {
                            if let Some(buffer) = self.app.get_buffer(buffer_id) {
                                if buffer.is_dirty() { " ●" } else { "" }
                            } else {
                                ""
                            }
                        } else {
                            ""
                        };

                        div()
                            .child(
                                format!(
                                    "Line {}, Column {} | UTF8 | No errors ✓{}",
                                    cursor_pos.line + 1,
                                    cursor_pos.column + 1,
                                    dirty_indicator
                                )
                            )
                            .text_color(rgb(0xffffff))
                            .text_size(px(12.0))
                    })
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = TypstEditor::new();
        assert_eq!(app.buffers.len(), 0);
    }

    #[test]
    fn test_create_buffer() {
        let mut app = TypstEditor::new();
        let id = app.create_buffer("Hello World");

        let buffer = app.get_buffer(id).unwrap();
        assert_eq!(buffer.text(), "Hello World");
    }

    #[test]
    fn test_create_window() {
        let mut app = TypstEditor::new();
        let window_id = app.new_window();
        assert_eq!(window_id, 0);
        assert_eq!(app.state.windows.len(), 1);
    }
}
