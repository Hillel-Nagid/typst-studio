# Native Cross-Platform Typst Editor - Detailed Specification

## Project Overview

A high-performance native text editor for Typst documents with bidirectional text support (RTL/LTR), live preview capabilities, and full LSP integration, built using GPUI framework.

## Technology Stack

- **UI Framework**: GPUI (Rust-based GPU-accelerated UI)
- **Language**: Rust
- **Text Processing**: Typst compiler + Typst LSP
- **Text Rendering**: Unicode bidirectional algorithm (UAX #9)
- **PDF Preview**: PDF.js or native rendering
- **LSP**: Typst's built-in language server

---

## Phase 1: Foundation & Architecture (Weeks 1-3)

### 1.1 Project Setup & Build Infrastructure

**Repository Structure:**

```
typst-editor/
├── crates/
│   ├── editor-core/           # Core editing logic
│   │   ├── buffer/            # Text buffer implementation
│   │   ├── selection/         # Selection and cursor management
│   │   └── operations/        # Edit operations (insert, delete, etc.)
│   ├── typst-integration/     # Typst compiler wrapper
│   │   ├── compiler/          # Compilation service
│   │   ├── diagnostics/       # Error handling
│   │   └── world/             # File system abstraction
│   ├── bidi-text/             # Bidirectional text handling
│   │   ├── algorithm/         # UAX #9 implementation
│   │   ├── layout/            # Visual layout engine
│   │   └── cursor/            # Bidi-aware cursor movement
│   ├── preview/               # Preview rendering
│   │   ├── renderer/          # PDF/SVG rendering
│   │   ├── sync/              # Source-preview synchronization
│   │   └── viewport/          # Viewport management
│   ├── lsp-client/            # LSP client implementation
│   │   ├── protocol/          # LSP protocol handlers
│   │   ├── requests/          # Request management
│   │   └── notifications/     # Notification handling
│   └── ui-components/         # Reusable UI components
│       ├── editor_view/       # Main editor component
│       ├── preview_pane/      # Preview component
│       ├── sidebar/           # File explorer, outline
│       └── panels/            # Bottom panels (errors, search)
├── assets/
│   ├── icons/                 # Application icons
│   ├── themes/                # Color schemes
│   └── fonts/                 # Bundled fonts
├── tests/
│   ├── integration/           # Integration tests
│   └── fixtures/              # Test documents
└── src/                       # Main application entry point
```

**Build Configuration:**

- **macOS**: Universal binary (Intel + Apple Silicon), code signing, notarization
- **Linux**: AppImage, Flatpak, and .deb packages, support for X11 and Wayland
- **Windows**: MSIX installer, portable executable, proper DPI awareness
- **CI/CD**: GitHub Actions for automated builds and releases
- **Dependencies**: Lock file management, security auditing, reproducible builds

**Development Environment:**

- Hot reload for UI development using GPUI's development features
- Logging infrastructure with multiple verbosity levels
- Debug overlays showing buffer state, bidi levels, LSP status
- Performance profiling integration with Tracy or similar tools
- Memory leak detection and sanitizers

### 1.2 Core Architecture Design

**Application State Management:**

The application uses a centralized state management approach with GPUI's reactive model:

1. **ApplicationState**:

   - Window management (multiple windows, workspace sessions)
   - Global settings and preferences
   - Recent files and projects
   - Theme and appearance state

2. **WorkspaceState**:

   - Active project root directory
   - Open files and editor tabs
   - Split pane layout configuration
   - Sidebar and panel visibility

3. **EditorState**:

   - Text buffer content and metadata
   - Cursor positions and selections (multiple cursors)
   - Scroll position and viewport
   - Syntax highlighting state
   - LSP diagnostics and decorations

4. **PreviewState**:
   - Rendered document (PDF/SVG)
   - Zoom level and scroll position
   - Synchronization markers
   - Compilation status

**Event Flow Architecture:**

```
User Input → GPUI Event System → Action Dispatcher → State Updates → Re-render
                                      ↓
                                  LSP Client → Typst LSP → Responses
                                      ↓
                                  Compiler → Preview Update
```

**Threading Model:**

1. **Main Thread**: UI rendering, user input handling, state updates
2. **LSP Thread**: Dedicated thread for LSP communication to prevent blocking
3. **Compilation Thread**: Background compilation with cancellation support
4. **File Watcher Thread**: Monitor filesystem changes
5. **Render Thread**: PDF/SVG rendering operations (may use thread pool)

**Inter-component Communication:**

- **Message Passing**: Use channels (tokio::mpsc) for async operations
- **Shared State**: Arc<RwLock<T>> for thread-safe shared data
- **Event Bus**: Centralized event system for loosely coupled components
- **Reactive Subscriptions**: Components subscribe to state changes

### 1.3 Data Models

**Document Model:**

- Unique document ID for tracking across the application
- File path and metadata (last modified, read-only status)
- Dirty flag for unsaved changes
- Document language (Typst, with potential for future languages)
- Encoding detection and handling (UTF-8 enforced for Typst)
- Line ending style (LF, CRLF, CR) detection and preservation

**Project Model:**

- Project root directory
- Main entry file detection (main.typ, index.typ, or user-specified)
- Dependency tracking (imported files, assets)
- Project-specific settings override
- Build output directory configuration
- Custom compiler arguments

**Configuration Model:**

- Hierarchical configuration: defaults → global → workspace → document
- Hot reload of configuration changes without restart
- Validation of configuration values with error reporting
- Import/export of configuration profiles
- Configuration schema for editor extensions

---

## Phase 2: Text Buffer with Bidirectional Support (Weeks 4-6)

### 2.1 Core Text Buffer Implementation

**Buffer Data Structure Specification:**

The text buffer must support:

- **Efficient random access**: O(log n) for most operations
- **Efficient line-based operations**: Quick line number to offset conversion
- **Efficient insertion/deletion**: Minimal copying on edits
- **Snapshot capability**: Immutable snapshots for async operations (LSP, compilation)
- **Unicode correctness**: All operations respect grapheme cluster boundaries

**Rope Structure Details:**

- Chunk size: 1024-4096 bytes for optimal cache performance
- B-tree variant with leaf nodes containing text chunks
- Internal nodes contain cumulative metrics:
  - Byte offset
  - Character count (Unicode scalar values)
  - Line count
  - Grapheme cluster count (for cursor positioning)

**Buffer Metrics Tracking:**

The buffer maintains real-time metrics:

- Total lines, characters, bytes
- Line lengths (for horizontal scrolling optimization)
- Longest line length (affects rendering decisions)
- Revision number (incremented on each edit, used for LSP)
- Modified ranges since last compilation (for incremental updates)

**Undo/Redo System:**

- **Operation History**: Each edit operation stores:

  - Operation type (insert, delete, replace)
  - Affected range (start, end positions)
  - Old content (for undo)
  - New content (for redo)
  - Cursor position before and after
  - Selection state
  - Timestamp

- **Grouping Strategy**:

  - Time-based: Operations within 1 second form a group
  - Semantic: Typing consecutive characters forms one undo step
  - Boundary detection: Whitespace, line breaks create boundaries
  - Explicit: User can force undo boundary (Ctrl+Z twice quickly)

- **Memory Management**:
  - History size limits (default: 1000 operations or 10MB)
  - Compact operations (consecutive single-char inserts → bulk insert)
  - Discard history for large operations (bulk replace over 1MB)

**Snapshot Mechanism:**

Snapshots are immutable views of the buffer at a point in time:

- Used by LSP for text synchronization without blocking edits
- Used by compiler to ensure consistent compilation
- Copy-on-write semantics: Share data with main buffer until divergence
- Snapshots include: full text content, revision number, document version
- Automatic cleanup when no longer referenced

### 2.2 Bidirectional Text Processing

**Unicode Bidirectional Algorithm Implementation:**

The editor must fully implement UAX #9 standard:

1. **Paragraph Detection**:

   - Paragraph boundaries: hard line breaks (LF, CRLF, CR, PS)
   - Empty lines are separate paragraphs
   - Each paragraph processed independently for bidi resolution

2. **Base Direction Determination**:

   - **Auto-detection**: Scan for first strong directional character
   - **Explicit override**: Support Unicode directional formatting characters:
     - RLE (U+202B): Right-to-left embedding
     - LRE (U+202A): Left-to-right embedding
     - RLO (U+202E): Right-to-left override
     - LRO (U+202D): Left-to-right override
     - PDF (U+202C): Pop directional formatting
     - RLI, LRI, FSI, PDI (U+2067-U+2069): Isolates
   - **User override**: Allow forcing paragraph direction via commands

3. **Embedding Level Calculation**:

   - Resolve character embedding levels (0-125)
   - Handle nested embeddings correctly
   - Process explicit directional controls
   - Resolve weak and neutral characters

4. **Visual Reordering**:
   - Reorder characters according to resolved levels
   - Create visual runs (consecutive characters at same level)
   - Handle line breaks: reorder within lines, not across
   - Mirror characters as needed (parentheses, brackets)

**Bidirectional Run Storage:**

For each paragraph, store:

```
BidiParagraph {
    base_direction: Direction (LTR/RTL),
    resolved_levels: Vec<u8>,           // Per-character embedding level
    visual_runs: Vec<VisualRun> {
        logical_range: Range<usize>,    // Range in original text
        visual_order: usize,            // Display order (0, 1, 2...)
        direction: Direction,           // Run direction
        level: u8,                      // Embedding level
    },
    reordered_indices: Vec<usize>,      // Map visual position → logical
    original_indices: Vec<usize>,       // Map logical → visual position
}
```

**Caching Strategy:**

- Cache bidi information per paragraph
- Invalidate cache only for modified paragraphs
- Lazy evaluation: compute bidi info only when needed for rendering
- Background computation for large documents: process visible paragraphs first
- Memory limit: discard cache for offscreen paragraphs in huge documents

**Mixed Direction Layout:**

The editor must handle complex scenarios:

- **Mixed scripts on same line**: Arabic and English in one paragraph
- **Numbers in RTL text**: Numbers should flow LTR even in RTL context
- **Punctuation handling**: Neutral characters inherit direction from context
- **Nested embeddings**: Support for multiple levels of direction changes

### 2.3 Cursor and Selection Logic

**Cursor Position Representation:**

Each cursor has dual representation:

- **Logical position**: (line, column) in source text, column in Unicode grapheme clusters
- **Visual position**: (line, visual_column) in rendered output
- **Affinity**: For bidi boundaries, affinity determines which direction the cursor prefers

**Cursor Movement Specification:**

1. **Horizontal Movement (Arrow Keys)**:

   **Left Arrow**:

   - In LTR context: Move to previous visual position
   - In RTL context: Move to next visual position
   - At direction boundary: Use cursor affinity to determine behavior
   - With Ctrl/Cmd: Move by word boundaries (respecting direction)

   **Right Arrow**:

   - In LTR context: Move to next visual position
   - In RTL context: Move to previous visual position
   - At direction boundary: Jump to the other side

   **Implementation details**:

   - Convert cursor logical position to visual position
   - Move in visual space
   - Convert back to logical position
   - Update affinity based on movement direction

2. **Vertical Movement (Up/Down Arrows)**:

   - Maintain "sticky" column: remember preferred visual column
   - Move to same visual column on target line
   - If target line is shorter, move to end of line
   - Preserve affinity when possible
   - Handle different line directions (LTR line to RTL line)

3. **Jump Movement (Home/End)**:

   **Home Key**:

   - First press: Move to first non-whitespace character (visual start of content)
   - Second press: Move to beginning of line (visual start)
   - In RTL line: These are on the right side

   **End Key**:

   - Move to end of line (visual end)
   - In RTL line: This is on the left side

   **Ctrl+Home/End**:

   - Move to beginning/end of document
   - Preserve logical ordering (not visual)

4. **Word Movement (Ctrl+Arrow)**:
   - Define word boundaries using Unicode word segmentation (UAX #29)
   - Respect script-specific rules (Arabic word boundaries differ from English)
   - Move to word boundary in the direction of movement
   - Handle punctuation and whitespace correctly

**Selection Handling:**

**Selection Representation:**

```
Selection {
    anchor: Position,        // Fixed point (where selection started)
    cursor: Position,        // Moving point (current cursor)
    affinity: Affinity,      // Direction preference at boundaries
    granularity: Granularity // Character, Word, Line, or Block
}
```

**Selection Types**:

1. **Character Selection** (default):

   - Extends character by character in visual order
   - Handles direction boundaries correctly
   - Shows selection highlight covering all characters from anchor to cursor

2. **Word Selection** (double-click, Ctrl+Shift+Arrow):

   - Extends selection by whole words
   - Snap to word boundaries
   - Expand to include entire word under cursor initially

3. **Line Selection** (triple-click):

   - Select entire lines including line break
   - Extend by lines with Shift+Up/Down

4. **Block Selection** (Alt+Drag):
   - Rectangular selection across multiple lines
   - Each line has independent start/end columns
   - Particularly useful for columnar data

**Selection Rendering:**

- **Visual spans**: Convert logical selection range to visual spans
- **Highlight rectangles**: One rectangle per visual span
- **Direction indicators**: Show small arrow at selection boundaries in mixed-direction text
- **Color**: Use theme-defined selection color with appropriate alpha
- **Multiple selections**: Support multiple simultaneous selections (like Sublime Text)

**Bidi Selection Edge Cases:**

1. **Selection crossing direction boundary**:

   - Split selection into multiple visual regions
   - Each region highlighted separately
   - Ensure continuous logical selection despite visual gaps

2. **Selection in nested embeddings**:

   - Calculate embedding levels for entire selection
   - Render highlight respecting visual reordering at each level

3. **Selection ending mid-word in opposite direction**:
   - Use affinity to determine if selection includes or excludes boundary character

**Mouse Selection Behavior:**

1. **Click**: Place cursor at clicked visual position
2. **Drag**: Create selection from click point to drag point (visual order)
3. **Double-click**: Select word at click point
4. **Triple-click**: Select line at click point
5. **Shift+Click**: Extend selection from cursor to click point
6. **Ctrl/Cmd+Click**: Add additional cursor/selection (multi-cursor)

### 2.4 Text Operations

**Insertion Operations:**

1. **Character Insertion**:

   - Respect bidi properties: inserted characters inherit direction from context
   - Update bidi information incrementally
   - Trigger syntax highlighting update
   - Notify LSP of text change
   - Add to undo history

2. **Paste Operation**:

   - Handle clipboard formats: plain text, rich text (extract plain)
   - Preserve line endings or normalize to document style
   - Handle large pastes efficiently (chunked operations)
   - Multi-cursor paste: distribute lines to cursors if count matches

3. **Auto-indentation**:
   - Analyze previous line indentation
   - Apply indentation rules based on Typst syntax
   - Handle tab vs. space preference
   - Smart indentation after braces, colons, etc.

**Deletion Operations:**

1. **Backspace**:

   - Delete previous grapheme cluster
   - In RTL context, this is the visual-left character
   - Handle deletion at direction boundaries correctly
   - Smart deletion: if deleting whitespace after indent, remove entire indent level

2. **Delete**:

   - Delete next grapheme cluster
   - In RTL context, this is the visual-right character

3. **Word Deletion (Ctrl+Backspace/Delete)**:

   - Delete previous/next word
   - Respect word boundaries in current direction

4. **Line Deletion**:
   - Delete entire line(s) containing cursor/selection
   - Preserve line break or join with next line

**Replacement Operations:**

- Atomic operation: delete selection + insert new text
- Used for find-replace, auto-correction, LSP edits
- More efficient than separate delete + insert for undo/redo

---

## Phase 3: GPUI Editor View (Weeks 7-9)

### 3.1 Editor View Component Hierarchy

**EditorView Structure:**

```
EditorView
├── Gutter
│   ├── LineNumbers
│   ├── FoldingMarkers
│   ├── Markers
│   └── GitDiff indicators
├── TextContent
│   ├── LineRenderer (per visible line)
│   │   ├── SyntaxHighlighting
│   │   ├── InlineWidgets (diagnostics, hints)
│   │   └── TextRunRenderer (handles bidi)
│   └── CursorRenderer
│       ├── PrimaryCursor (blinking)
│       └── SecondaryCursors (multi-cursor)
├── ScrollBar
│   ├── Vertical (with minimap mode)
│   └── Horizontal
├── Overlays
│   ├── Autocomplete popup
│   ├── Hover information
│   ├── Parameter hints
│   └── Quick fixes menu
└── StatusBar
    ├── Position indicator (line:col)
    ├── Selection info (chars selected)
    ├── Encoding display
    └── Language mode
```

**Layout System:**

- **Flex layout**: Use GPUI's flex system for component arrangement
- **Sizing constraints**: Minimum/maximum sizes for panes
- **Overflow handling**: Scrolling regions with proper clipping
- **Z-ordering**: Overlays above content, cursors above text, etc.

### 3.1.1 Menu System (NEW)

**Menu Bar Architecture:**

- Top-level menu items: File, Edit, View, Compile, Help
- Dropdown menus with keyboard shortcuts
- Platform-specific menu behavior (macOS menubar vs Windows/Linux in-window)

**Menu Actions (Phase 1 Implementation - Placeholder UI):**

- **File**: New, Open, Save, Save As, Close, Exit
- **Edit**: Undo, Redo, Cut, Copy, Paste, Find, Replace
- **View**: Toggle Sidebar, Toggle Preview, Zoom In/Out, Toggle Theme
- **Compile**: Compile Document, Export PDF, Export PNG
- **Help**: Documentation, Keyboard Shortcuts, About

**Implementation Status:**
- ✓ Phase 1: Menu structure and basic UI rendering
- → Phase 2: Wire up file operations (open, save, close)
- → Phase 3: Integrate edit operations and compile features

### 3.1.2 Top Navigation Bar (NEW)

**TopNav Components:**

- Logo/branding section: "T" icon (blue) + "Typst Studio" text
- Integrated MenuBar with File, Edit, View, Compile, Help
- Right-side toolbar:
  - Search icon (placeholder)
  - Save icon (placeholder)
  - Hamburger menu icon (placeholder)
  - Close button (closes window)

**Visual Styling:**

- Background: #2d2d30 (dark gray)
- Height: 36px
- Text color: #cccccc (light gray)
- Hover state: #3e3e42 (slightly lighter)
- Close button: Blue (#007acc) background with white icon

### 3.1.3 Split-Pane Layout (NEW)

**Default Layout Configuration:**

- Vertical split: Editor pane (left) | Preview pane (right)
- Default ratio: 50/50
- Adjustable splitter with drag handle (1px divider)
- Minimum pane width: 300px
- Panel labels: "EDITOR" and "PREVIEW" at top of each pane

**Pane Headers:**

- Height: 32px
- Background: #2d2d30
- Text: "EDITOR" or "PREVIEW" (bold, 12px)
- Additional info in preview pane (right-aligned)

**Layout Persistence:**

- Remember split ratio per workspace (future enhancement)
- Remember which panes are visible (future enhancement)
- Restore layout on application restart (future enhancement)

### 3.2 Text Rendering Pipeline

**Rendering Stages:**

1. **Text Shaping**:

   - Use HarfBuzz (via rustybuzz) for complex text shaping
   - Handle ligatures, kerning, OpenType features
   - Script-specific shaping (Arabic joining, Devanagari combining marks)
   - Emoji rendering (color emoji support)
   - Cache shaped runs for identical text

#### 3.2.1 Bidirectional Text Processing (Implementation Complete)

**Unicode Bidirectional Algorithm Integration (UAX #9):**

The editor now fully implements the UAX #9 standard through the `bidi-text` crate, enabling proper mixed RTL/LTR text rendering on the same line.

**Architecture:**

1. **TextShaper Enhancement** (`crates/ui-components/src/rendering/text_shaping.rs`):
   - New `shape_with_bidi()` method processes text through Unicode Bidi Algorithm
   - Returns `BidiShapedText` containing multiple shaped runs with direction info
   - Each run (`BidiShapedRun`) preserves:
     - Logical range in original text
     - Direction (LTR/RTL)
     - Embedding level (for nested bidi)
     - Shaped glyphs

2. **LineLayout Enhancement** (`crates/ui-components/src/rendering/line_layout.rs`):
   - New `compute_visual_lines_with_bidi()` method accepts `BidiShapedText`
   - Calculates x_offset for each run in visual order
   - LTR base: runs flow left-to-right from position 0
   - RTL base: runs flow right-to-left from right edge
   - Produces `VisualLine` with multiple `VisualTextRun` entries

3. **LineRenderer Integration** (`crates/ui-components/src/editor_view/line_renderer.rs`):
   - Uses `TextShaper.shape_with_bidi()` for each line
   - Caches bidi-shaped results for performance
   - Processes lines through full bidi+shaping pipeline

4. **EditorView Integration** (`crates/ui-components/src/editor_view/mod.rs`):
   - Updated rendering pipeline to support mixed RTL/LTR
   - Sample text demonstrates bidi support: "// Mixed text: English אבג 123 عرب"
   - Status bar indicates "RTL/LTR: Enabled"

**Data Flow:**

```
Buffer Line Text
    ↓
BidiParagraph (UAX #9 algorithm)
    ↓ (visual runs with direction)
TextShaper.shape_with_bidi()
    ↓ (BidiShapedText with shaped glyphs per run)
LineLayout.compute_visual_lines_with_bidi()
    ↓ (VisualLine with positioned runs)
Renderer
    ↓
Screen
```

**Supported Features:**

- ✅ Mixed RTL/LTR text on same line
- ✅ Auto-detection of base direction from first strong character
- ✅ Proper handling of neutral characters (numbers, punctuation)
- ✅ Numbers flow LTR even in RTL context
- ✅ Multiple embedding levels (nested bidi)
- ✅ Efficient caching of shaped runs

**Test Results:**

All bidi-text crate tests pass:
- ✅ LTR text rendering (English)
- ✅ RTL text rendering (Hebrew: שלום)
- ✅ Mixed text handling (Hello שלום World - multiple runs)
- ✅ Cursor movement across direction boundaries
- ✅ Visual line layout calculations

2. **Font Management**:

   - Font fallback chain: primary font → system fonts → default
   - Per-script font selection (Arabic text may use different font than Latin)
   - Font variations: weight, style, stretch
   - Font features: ligatures on/off, stylistic alternates
   - Font metrics: ascent, descent, line gap for line height calculation

3. **Glyph Positioning**:

   - Calculate baseline position for each line
   - Position glyphs according to shaping results
   - Apply subpixel positioning for smooth rendering (if GPU supports)
   - Handle combining characters (diacritics stack correctly)

4. **Line Layout**:
   - Measure line width accounting for all glyphs and bidi reordering
   - Word wrapping: break at whitespace, respect word boundaries
   - Soft wrap indicators (visual markers for wrapped lines)
   - Horizontal scrolling for long lines (when wrap disabled)

**Visual Line Calculation:**

For each logical line, calculate visual lines (wrapped):

```
VisualLine {
    logical_line: usize,               // Source line number
    visual_line_index: usize,          // Index within wrapped line (0, 1, 2...)
    char_range: Range<usize>,          // Characters from logical line in this visual line
    pixel_width: f32,                  // Rendered width
    baseline_y: f32,                   // Vertical position
    bidi_runs: Vec<VisualTextRun>,    // Reordered text segments
}

VisualTextRun {
    text: String,                      // Actual text content
    direction: Direction,
    x_offset: f32,                     // Horizontal position
    glyphs: Vec<ShapedGlyph>,         // Shaped and positioned glyphs
    style: TextStyle,                  // Color, font, decorations
}
```

**Viewport Management:**

- **Visible lines**: Calculate which logical lines are in viewport
- **Render padding**: Render extra lines above/below for smooth scrolling (2-3 lines)
- **Virtual scrolling**: For huge documents, render only visible portion
- **Smooth scrolling**: Pixel-perfect scrolling rather than line-based
- **Scroll anchoring**: Keep specific line at specific position during edits

### 3.3 Syntax Highlighting

**Highlighting Architecture:**

1. **Parser Integration**:

   - Use typst-syntax for parsing
   - Incremental parsing: re-parse only changed portions
   - Error recovery: continue parsing after syntax errors
   - Query language for highlight patterns

2. **Token Classification**:

   - Keywords: #import, #let, #set, #show, etc.
   - Functions: built-in and user-defined
   - Variables and parameters
   - Strings and interpolations
   - Comments: line and block
   - Markup: headings, emphasis, links
   - Math mode delimiters and content
   - Raw/code blocks
   - Labels and references

3. **Semantic Highlighting** (via LSP):

   - Variable references colored by type
   - Function calls colored by return type
   - Unused variables grayed out
   - Deprecated items marked with strikethrough
   - Error tokens underlined in red

4. **Theme System**:
   - Token scopes map to theme colors
   - Support for TextMate-style themes
   - Dark and light mode variants
   - User-customizable themes
   - Live theme preview

**Typst Studio Dark Theme (NEW - Phase 1 Implementation):**

Color Palette:

- **UI Colors**:
  - Window background: #2d2d30
  - Editor background: #1e1e1e
  - Gutter background: #252526
  - Text (normal): #cccccc
  - Text (dim): #858585
  - Cursor: #cccccc
  - Selection: #264f78

- **Syntax Colors**:
  - Keywords (#import, #let, #set, #show): #569cd6 (blue)
  - Functions (user-defined/built-in): #c586c0 (purple)
  - Strings and interpolations: #ce9178 (orange)
  - Comments (line and block): #6a9955 (green)
  - Numbers and constants: #b5cea8 (light green)
  - Operators: #cccccc (normal text)
  - Type names: #4ec9b0 (teal)
  - Error: #f44444 (red)
  - Warning: #ff9933 (orange)

- **UI Element Colors**:
  - Status bar background: #007acc (blue)
  - Status bar text: #ffffff (white)
  - Menu bar background: #2d2d30
  - Menu hover: #3e3e42
  - Button background: #2d2d30
  - Button hover: #3e3e42

**Implementation Status**:
- ✓ Phase 1: Typst Studio Dark theme created and set as default
- → Phase 2: Light theme variant
- → Phase 3: Theme switching UI

**Highlighting Performance:**

- Syntax highlighting runs asynchronously
- Visible lines prioritized over offscreen
- Incremental updates on edit: only re-highlight affected ranges
- Caching of highlight results keyed by buffer revision
- Budget-based highlighting: stop after 16ms if not complete, continue next frame

### 3.4 Input Handling

**Keyboard Input Processing:**

**Text Input:**

- Use GPUI's text input events for character input
- IME (Input Method Editor) support for CJK languages:
  - Show composition preview underline
  - Allow user to commit or cancel composition
  - Handle IME candidate selection window
- Dead key support for diacritics (e.g., ´ + e = é)
- Compose key sequences (Linux)

**Key Bindings:**

Implement comprehensive key binding system:

- Default keymaps for each platform (Mac, Windows, Linux conventions)
- User-customizable key bindings via config file
- Multi-key sequences (e.g., Ctrl+K, Ctrl+C for comment)
- Context-sensitive bindings (different actions in different modes)
- Vim/Emacs keybinding emulation modes (optional)

**Standard Editing Commands:**

- Cursor movement: Arrow keys, Home/End, Page Up/Down
- Selection: Shift+movement keys
- Clipboard: Ctrl/Cmd+C/X/V
- Undo/Redo: Ctrl/Cmd+Z/Y
- Save: Ctrl/Cmd+S
- Find: Ctrl/Cmd+F
- Multi-cursor: Ctrl/Cmd+D (select next occurrence)

**Mouse Input:**

**Click Actions:**

- Single click: position cursor
- Double click: select word
- Triple click: select line
- Quadruple click: select all
- Shift+click: extend selection
- Ctrl/Cmd+click: add cursor (multi-cursor mode)
- Alt+drag: block selection

**Scrolling:**

- Mouse wheel: vertical scroll
- Shift+wheel: horizontal scroll
- Ctrl/Cmd+wheel: zoom in/out (change font size)
- Trackpad gestures: two-finger scroll, pinch-to-zoom

**Hover:**

- Hover over symbols: show LSP hover information after 500ms delay
- Hover over errors: show diagnostic details
- Hover over links: show URL and make clickable

**Touch Input (for tablets):**

- Single tap: position cursor
- Long press: show context menu
- Two-finger drag: scroll
- Pinch: zoom

### 3.5 Decorations and Annotations

**Inline Decorations:**

1. **Error Squiggles**:

   - Red wavy underline for errors
   - Yellow wavy underline for warnings
   - Blue wavy underline for information
   - Gray wavy underline for hints
   - Hover shows full diagnostic message

2. **Code Lens**:

   - Above functions: show type signature, documentation link
   - Above variables: show inferred type
   - Clickable to perform actions (go to definition, find references)

3. **Inline Hints**:

   - Parameter names in function calls (when multiple args)
   - Inferred types for variables without explicit type
   - Shown in grayed-out text, can be toggled off

4. **Brackets and Braces**:
   - Highlight matching bracket when cursor adjacent
   - Rainbow brackets: nested pairs get different colors
   - Show matching bracket in gutter when off-screen

**Gutter Decorations:**

1. **Line Numbers**:

   - Show absolute line numbers by default
   - Option for relative line numbers (Vim-style)
   - Highlight current line number
   - Dim line numbers for wrapped continuation lines

2. **Folding Markers**:

   - Triangle icons to fold/unfold code sections
   - Based on indentation or syntax tree
   - Fold entire functions, sections, etc.
   - Show summary of folded content on hover

3. **Diagnostic Markers**:

   - Icon in gutter for lines with errors/warnings
   - Color-coded by severity
   - Click to show details or navigate to next/previous

4. **Git Diff Indicators**:
   - Green bar: added lines
   - Blue bar: modified lines
   - Red triangle: deleted lines (shown above)
   - Click to show diff popup

**Highlight Ranges:**

- Current line highlight: subtle background color
- Selection highlight: theme-defined color
- Search result highlights: yellow background
- Write occurrences: when cursor on symbol, highlight all writes in subtle color
- Read occurrences: highlight all reads in different subtle color

---

## Phase 4: LSP Integration (Weeks 10-12)

### 4.1 LSP Client Architecture

**LSP Communication Layer:**

**Protocol Implementation:**

- Full LSP 3.17 specification support
- JSON-RPC 2.0 over stdio (Typst LSP runs as subprocess)
- Message framing: Content-Length header parsing
- Request/response correlation using message IDs
- Notification handling (no response expected)
- Error handling and recovery

**Client Lifecycle:**

1. **Initialization**:

   - Start Typst LSP subprocess with appropriate arguments
   - Send `initialize` request with client capabilities:
     - Text document sync (incremental)
     - Completion support (with snippet support)
     - Hover support
     - Signature help
     - Go to definition/declaration/implementation
     - Find references
     - Document highlighting
     - Document symbols
     - Code actions
     - Code lens
     - Formatting
     - Rename
     - Folding ranges
     - Inlay hints
   - Receive server capabilities in response
   - Send `initialized` notification
   - Send `workspace/didChangeConfiguration` with settings

2. **Document Synchronization**:

   - Send `textDocument/didOpen` when file opened
   - Send `textDocument/didChange` on edits (incremental changes only)
   - Include document version number (increments on each change)
   - Debounce changes: send updates at most every 100ms
   - Send `textDocument/didSave` after save
   - Send `textDocument/didClose` when document closed

3. **Shutdown**:
   - Send `shutdown` request
   - Wait for response
   - Send `exit` notification
   - Terminate subprocess gracefully
   - Handle crashes: restart LSP and resynchronize documents

**Request Management:**

**Request Queue:**

- Prioritize user-facing requests (hover, completion) over background (diagnostics)
- Cancel pending requests when new conflicting request arrives (e.g., new hover cancels old)
- Timeout requests after 5 seconds, show error to user
- Track pending requests for debugging and telemetry

**Response Handling:**

- Deserialize JSON responses using serde
- Validate response structure (handle missing optional fields)
- Update UI based on response (e.g., show completion list)
- Error handling: show error message, log for diagnostics

### 4.2 LSP Features Implementation

**Diagnostics (Errors and Warnings):**

**Receiving Diagnostics:**

- Server sends `textDocument/publishDiagnostics` notifications
- Contains URI, version, and list of diagnostics
- Each diagnostic has:
  - Range (start/end position)
  - Severity (error, warning, info, hint)
  - Message text
  - Optional code and source
  - Optional related information (other locations)
  - Optional code actions (quick fixes)

**Displaying Diagnostics:**

- Underline text in editor with wavy line (color by severity)
- Show icon in gutter
- Add to problems panel (sortable, filterable list)
- Tooltip on hover shows full message
- Navigate with F8/Shift+F8 (next/previous diagnostic)

**Code Actions:**

- Request available actions via `textDocument/codeAction`
- Show lightbulb icon in gutter when actions available
- Display menu of actions on click or keyboard shortcut
- Actions may include:
  - Quick fixes for diagnostics
  - Refactorings (extract variable, inline, etc.)
  - Source actions (organize imports, format document)

**Completion:**

**Triggering Completion:**

- Automatic trigger after typing (configurable delay: 100-300ms)
- Trigger characters: `.` for method completion, `#` for Typst commands
- Manual trigger: Ctrl+Space

**Request:**

- Send `textDocument/completion` with cursor position
- Include completion context (trigger kind and character)

**Response Handling:**

- Receive list of completion items, each with:
  - Label (displayed text)
  - Kind (function, variable, keyword, etc.)
  - Detail (type signature or brief description)
  - Documentation (full description, may be markdown)
  - Insert text or text edit
  - Sort text (for ordering)
  - Filter text (for matching)
  - Additional text edits (e.g., add import)
  - Command to execute after insert (optional)

**Completion UI:**

- Show popup below/above cursor (depending on space)
- Fuzzy matching: filter items as user types
- Highlight matched characters
- Show item kind icon
- Show signature/detail in right column
- Up/down arrows to navigate, Enter/Tab to accept
- Escape to dismiss
- Show documentation panel on side for selected item

**Snippet Support:**

- Parse snippet syntax (LSP snippet format)
- Tabstops: $1, $2, etc. for cursor positions
- Placeholders: ${1:default} with default text
- Choices: ${1|option1,option2|}
- Variables: $NAME or ${NAME:default}
- Tab to next tabstop, Shift+Tab to previous

**Hover Information:**

**Request:**

- Send `textDocument/hover` when user hovers over symbol
- Debounce: wait 300ms before sending to avoid excessive requests

**Response:**

- Receive markdown content and optional range
- Content may include:
  - Type information
  - Documentation
  - Code examples
  - Links to documentation

**Display:**

- Show in floating popup near cursor
- Render markdown (headings, bold, italic, code blocks)
- Syntax highlight code blocks in hover
- Make links clickable (open in browser or navigate to document)
- Dismiss on mouse move away or Escape key
- Resize popup based on content size (max width/height limits)

**Signature Help (Parameter Hints):**

**Triggering:**

- Automatic: when typing function call opening parenthesis
- Trigger characters: `(` and `,`
- Manual: Ctrl+Shift+Space

**Request:**

- Send `textDocument/signatureHelp` with cursor position
- Include signature help context (trigger kind, active signature)

**Response:**

- List of overloaded signatures (if function has multiple variants)
- Each signature contains:
  - Label (full function signature)
  - Documentation
  - Parameters list with labels, documentation, and active ranges
  - Active parameter index

**Display:**

- Floating popup above current line
- Show all overloads with navigation arrows if multiple
- Highlight active parameter in bold
- Show parameter documentation below signature
- Update active parameter as user types commas
- Auto-dismiss when closing parenthesis typed

**Go To Definition/Declaration/Implementation:**

**Request:**

- Send `textDocument/definition` (or declaration/implementation)
- Include cursor position

**Response:**

- Single location or list of locations (for multiple definitions)
- Each location has URI and range

**Navigation:**

- Jump to definition in current editor if same file
- Open new tab if different file
- Show peek definition popup (inline preview) if configured
- Breadcrumb trail to navigate back (Ctrl+-)
- History of navigation jumps maintained

**Find References:**

**Request:**

- Send `textDocument/references`
- Include context: whether to include declaration

**Response:**

- List of locations where symbol is referenced

**Display:**

- Show in bottom panel with grouped results
- Group by file, show file path and line numbers
- Preview of each reference with highlighting
- Click to navigate to reference
- Keyboard navigation through results

**Document Symbols (Outline):**

**Request:**

- Send `textDocument/documentSymbol` for current file
- Triggered on file open and after edits (debounced)

**Response:**

- Hierarchical list of symbols:
  - Name
  - Kind (function, variable, heading, etc.)
  - Range in document
  - Children symbols (nested structures)

**Display:**

- Sidebar panel showing document outline
- Tree view with expand/collapse
- Icons for different symbol kinds
- Click to navigate to symbol
- Search/filter symbols
- Breadcrumbs at top of editor showing current symbol hierarchy

**Workspace Symbols:**

**Request:**

- Send `workspace/symbol` with search query
- Triggered by Ctrl+T or Cmd+T

**Response:**

- List of symbols across entire workspace matching query

**Display:**

- Quick-open popup with fuzzy search
- Show symbol name, kind, file path
- Navigate with arrows, Enter to open
- Incremental search as user types

**Rename Symbol:**

**Request:**

- Send `textDocument/rename` with position and new name
- Show input box to user first for new name

**Response:**

- Workspace edit containing changes across all files
- Changes include text edits with ranges

**Execution:**

- Preview changes in diff view before applying
- Allow user to review and uncheck unwanted changes
- Apply all edits atomically (all or nothing)
- Add to undo history as single operation

**Document Formatting:**

**Request:**

- Send `textDocument/formatting` for whole document
- Or `textDocument/rangeFormatting` for selection

**Response:**

- List of text edits to apply

**Execution:**

- Apply edits preserving cursor position where possible
- Format on save (if configured)
- Format on paste (if configured)
- Format on type (for specific characters like `}`)

**Inlay Hints:**

**Request:**

- Send `textDocument/inlayHint` with visible range
- Update when viewport changes

**Response:**

- List of hints with:
  - Position in document
  - Label (text to display)
  - Kind (type, parameter)
  - Tooltip (optional explanation)
  - Padding before/after

**Display:**

- Render inline in editor in dimmed color
- Parameter names: before arguments
- Type hints: after variable declarations
- Toggle visibility with command
- Clickable to perform action or show details

**Semantic Tokens:**

**Request:**

- Send `textDocument/semanticTokens/full` for initial tokens
- Send `textDocument/semanticTokens/full/delta` for updates

**Response:**

- Encoded token data with positions, lengths, and types
- Token types: namespace, type, class, enum, interface, struct, typeParameter, parameter, variable, property, enumMember, event, function, method, macro, keyword, modifier, comment, string, number, regexp, operator

**Integration:**

- Override or augment typst syntax highlighting
- More accurate coloring based on semantic analysis
- Unused symbols dimmed
- Deprecated symbols with strikethrough

**Code Lens:**

**Request:**

- Send `textDocument/codeLens` for visible range

**Response:**

- List of code lenses with:
  - Range to display above
  - Command to execute on click
  - Title/label

**Display:**

- Clickable text above code elements
- Examples: "5 references", "Run test", "Debug"
- Execute command when clicked
- Resolve lazy lenses on demand (second request for details)

**Folding Ranges:**

**Request:**

- Send `textDocument/foldingRange`

**Response:**

- List of ranges that can be folded
- Kind: comment, imports, region

**Integration:**

- Use to determine foldable sections
- Combine with indentation-based folding
- Show fold markers in gutter

### 4.3 LSP Configuration and Settings

**Typst LSP Specific Settings:**

**Compiler Settings:**

- Root directory for project
- Main entry file (if not auto-detected)
- Font paths for custom fonts
- Package cache directory
- Compilation mode (debug/release)

**Editor Settings:**

- Semantic highlighting enable/disable
- Inlay hints configuration:
  - Show parameter names (always, never, on hover)
  - Show type hints (always, never, on hover)
- Code lens enable/disable
- Format on save/type/paste

**Diagnostic Settings:**

- Diagnostic severity levels to show
- Maximum number of diagnostics per file
- Debounce delay for diagnostic updates

**Performance Settings:**

- Incremental compilation enable/disable
- Cache size limits
- Background indexing priority

**Export Settings:**

- PDF export options (passed to Typst)
- PNG export resolution
- SVG export settings

**Configuration Synchronization:**

- Send initial configuration on LSP start
- Watch configuration file for changes
- Send `workspace/didChangeConfiguration` when settings change
- Allow per-workspace configuration overrides
- Configuration schema validation before sending

### 4.4 LSP Error Handling and Recovery

**Connection Issues:**

**Subprocess Failures:**

- Detect if LSP process crashes or exits unexpectedly
- Show notification to user: "Typst language server stopped"
- Automatic restart with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- After 5 failed restarts, stop trying and ask user to check logs
- Button to manually restart LSP

**Communication Errors:**

- Malformed JSON: log error, ignore message, continue
- Timeout: cancel request, show warning for user-initiated actions
- Protocol errors: log, show notification with details

**State Desynchronization:**

**Detection:**

- Track document versions sent vs. received in diagnostics
- If mismatch detected, resynchronize

**Recovery:**

- Close all documents with `textDocument/didClose`
- Reopen all documents with `textDocument/didOpen` (full content)
- Re-send configuration
- Log incident for debugging

**Performance Degradation:**

**Monitoring:**

- Track request latency (completion, hover, etc.)
- Warn if LSP response times exceed thresholds (>1s for completion)

**Mitigation:**

- Increase debounce delays
- Reduce frequency of background requests
- Suggest user close large files or split workspace
- Show spinner/progress indicator for slow operations

**Graceful Degradation:**

If LSP unavailable:

- Disable LSP-dependent features gracefully
- Fall back to syntax highlighting only (typst-syntax)
- Show message in status bar: "Language features unavailable"
- Allow manual retry

---

## Phase 5: Typst Compilation and Preview (Weeks 13-15)

### 5.1 Compilation Pipeline Architecture

**Compiler Service Design:**

**Compilation Request Flow:**

```
Editor Change → Debouncer → Compilation Queue → Compiler Thread → Result Handler → Preview Update
                               ↓
                          Cancellation Signal (if new request arrives)
```

**Compilation Modes:**

1. **Incremental Compilation** (default):

   - Only recompile changed parts of document
   - Requires Typst's incremental compilation support
   - Fastest for small edits
   - Used during active editing

2. **Full Compilation**:

   - Recompile entire document from scratch
   - Triggered on: file save, project settings change, dependencies update
   - Slower but ensures correctness
   - Used as fallback if incremental fails

3. **Export Compilation**:
   - Full compilation with specific output format
   - Higher quality settings (no optimization shortcuts)
   - Used for final PDF export

**Compilation Context (Typst World):**

The compiler needs a "world" implementation:

```
SystemWorld {
    root: PathBuf,                          // Project root directory
    main: PathBuf,                          // Main entry file
    library: Prehashed<Library>,            // Typst standard library
    book: Prehashed<FontBook>,              // Available fonts
    fonts: Vec<FontSlot>,                   // Font data
    hashes: Mutex<HashMap<PathBuf, FileResult<PathHash>>>,  // File hashes cache
    paths: Mutex<HashMap<PathBuf, FileResult<PathBuf>>>,    // Path resolution cache
    sources: Mutex<HashMap<PathBuf, FileResult<Source>>>,   // Source file cache
    files: Mutex<HashMap<PathBuf, FileResult<Bytes>>>,      // Binary file cache
}
```

**File System Abstraction:**

- Virtual file system layer to handle:
  - Real files on disk
  - In-memory buffers for unsaved changes
  - Package downloads and caching
  - Network resources (images, data)

**Priority:**

1. In-memory buffer (if file is open in editor)
2. Disk cache (if file was recently read)
3. Fresh read from disk

**Dependency Tracking:**

- Track which files are imported by main document
- Track which assets are referenced (images, data files)
- Watch all dependencies for changes
- Recompile when dependency changes detected
- Show dependency graph in UI (optional debugging view)

### 5.2 Compilation Process Details

**Compilation Steps:**

1. **Pre-compilation Validation**:

   - Check if main file exists and is readable
   - Validate project structure
   - Ensure fonts are available
   - Check package dependencies are downloaded

2. **Source Preparation**:

   - Gather all source files (main + imports)
   - Create virtual file system with in-memory buffers
   - Resolve relative imports and paths
   - Set up font search paths

3. **Typst Compilation**:

   - Create Typst `World` instance
   - Parse source files into syntax tree
   - Type checking and semantic analysis
   - Layout calculation (flow text, positioning)
   - Render to output format (PDF, SVG, PNG)

4. **Post-compilation Processing**:

   - Extract diagnostics and warnings
   - Build source map (source positions → output positions)
   - Calculate page boundaries
   - Generate thumbnail previews (optional)

5. **Result Delivery**:
   - Send compiled document to preview renderer
   - Update diagnostics in editor
   - Update outline/structure view
   - Notify user if compilation failed

**Error Recovery:**

- Partial compilation: if possible, compile what can be compiled
- Show last successful output if current compilation fails
- Preserve diagnostics from failed compilation
- Continue allowing edits during compilation failure
- Show inline error at failure point in editor

**Compilation Optimization:**

**Caching Strategy:**

- Cache parsed syntax trees for unchanged files
- Cache font metrics and glyph data
- Cache resolved imports and file hashes
- Cache layout results for unchanged sections
- Invalidate cache intelligently on changes

**Incremental Recompilation:**

- Detect minimal change set (which lines/paragraphs changed)
- Reuse layout for unchanged sections
- Only re-layout affected pages
- Requires tracking dependencies between elements

**Parallel Processing:**

- Parse multiple imported files in parallel
- Render independent pages in parallel (for multi-page documents)
- Use thread pool sized to CPU cores
- Careful with shared state (use Arc/Mutex where needed)

**Debouncing and Throttling:**

**Debouncing** (wait for pause in edits):

- Start timer on first edit: 300ms default
- Reset timer on each subsequent edit
- Compile when timer expires
- Configuration: adjustable delay (100ms-2s)

**Throttling** (limit compilation frequency):

- Maximum compilation rate: once per 100ms
- Queue compilations if rate exceeded
- Drop intermediate compilations if queue grows (keep only latest)

**Adaptive Debouncing:**

- Longer debounce for slow compilations (if last compile took >1s, wait 500ms)
- Shorter debounce for fast compilations (if last compile took <100ms, wait 200ms)
- Learn from compilation times and adjust

### 5.3 Preview Rendering

**Rendering Backend Options:**

**Option A: PDF Rendering (via pdfium)**:

- **Pros**:
  - High fidelity to final output
  - Mature rendering engine
  - Handles complex documents well
- **Cons**:
  - Slower to render
  - Larger file sizes
  - More memory usage

**Option B: SVG Rendering**:

- **Pros**:
  - Faster rendering for simple documents
  - Smooth zooming (vector graphics)
  - Easier to implement source mapping
- **Cons**:
  - May not match PDF output exactly
  - Complex documents can have huge SVG files
  - Browser SVG rendering limitations

**Recommended: Hybrid Approach**:

- Use SVG for real-time preview during editing
- Switch to PDF for final review or on user request
- Allow user to toggle between modes
- Automatically switch to PDF if SVG becomes too large (>10MB)

**Preview Rendering Pipeline:**

1. **Document Reception**:

   - Receive compiled output from compilation service
   - Detect format (PDF, SVG, or both)
   - Calculate document dimensions and page count

2. **Page Rendering**:

   - Render visible pages only (viewport culling)
   - Render adjacent pages for smooth scrolling (1-2 pages buffer)
   - Use texture caching for rendered pages
   - Discard offscreen page textures to save memory

3. **Display Rendering**:

   - Composite pages with proper spacing
   - Apply zoom transformation
   - Render page shadows/borders
   - Show page numbers overlaid

4. **Interactive Elements**:
   - Clickable links (internal references, external URLs)
   - Hover highlights for interactive elements
   - Click to select text (if supported)

**Zoom and Pan:**

**Zoom Levels:**

- Fit width: scale to fit page width in viewport
- Fit page: scale to fit entire page in viewport
- Fit height: scale to fit page height
- Custom: 10%, 25%, 50%, 75%, 100%, 125%, 150%, 200%, 400%
- Smooth zoom with Ctrl+wheel (continuous scaling)

**Zoom Implementation:**

- Re-render pages at new resolution when zoom changes significantly
- Use GPU scaling for smooth intermediate zooms
- Maintain sharp rendering at all zoom levels (render at target resolution)

**Pan/Scroll:**

- Smooth pixel-perfect scrolling
- Momentum scrolling (inertial scrolling on trackpads)
- Keyboard: Page Up/Down, Home/End, Arrow keys
- Mouse: drag with middle button or with hand tool

**Page Layout Modes:**

1. **Single Page**: One page at a time, scroll to change pages
2. **Continuous**: All pages in vertical strip
3. **Two-Up**: Two pages side-by-side (like book spread)
4. **Two-Up Continuous**: Pairs of pages in vertical strip

### 5.4 Source-Preview Synchronization

**Forward Sync (Source → Preview):**

**Implementation:**

- Use Typst's source mapping feature to get output positions
- When user clicks in editor:
  - Get cursor position (line, column)
  - Query compilation result for corresponding output position
  - Calculate page and position within page
  - Scroll preview to show that position
  - Highlight corresponding region in preview (with animated pulse)

**Accuracy Considerations:**

- Source position may map to multiple output positions (repeated content)
- Show first occurrence or closest to current preview scroll position
- Handle unmapped positions gracefully (show beginning of page/document)

**Backward Sync (Preview → Source):**

**Implementation:**

- When user clicks in preview:
  - Get click coordinates (page, x, y)
  - Query compilation result for corresponding source position
  - If multiple source positions, show disambiguation menu
  - Scroll editor to show source position
  - Highlight corresponding line in editor (with animated pulse)

**Source Mapping Data Structure:**

```
SourceMapping {
    source_to_preview: HashMap<SourceSpan, Vec<PreviewLocation>>,
    preview_to_source: HashMap<PreviewLocation, Vec<SourceSpan>>,
}

SourceSpan {
    file: PathBuf,
    start: (usize, usize),  // (line, column)
    end: (usize, usize),
}

PreviewLocation {
    page: usize,
    bounds: Rectangle,  // (x, y, width, height) in page coordinates
}
```

**Sync Indicators:**

- Show brief highlight flash in destination when syncing
- Arrow or line connecting source and preview during sync
- Sync icon in gutter showing mappable regions
- Tooltip showing preview snippet when hovering over source

**Auto-sync Options:**

- Sync on click (default)
- Sync on cursor move (updates preview highlight as you navigate)
- Sync on scroll (preview follows editor scrolling)
- Sync disabled (manual sync only with keyboard shortcut)

### 5.5 Preview UI Components

**Preview Toolbar:**

**Zoom Controls:**

- Zoom in/out buttons (+ -)
- Zoom percentage dropdown with presets
- Fit width / Fit page buttons
- Reset zoom button (100%)

**Page Navigation:**

- Previous/next page buttons
- Page number input: "Page X of Y"
- Thumbnail navigation (show all pages as thumbnails)
- Jump to page command (Ctrl+G)

**View Options:**

- Layout mode selector (single, continuous, two-up)
- Rotate page (90° increments)
- Night mode (invert colors)
- Show/hide rulers and guides

**Export:**

- Export to PDF (save compiled output)
- Export current page as PNG
- Copy page as image
- Print preview and print

**Preview Settings:**

- Rendering quality (draft, normal, high)
- Background color
- Page margins/spacing
- Antialiasing settings

**Status Bar:**

- Current page / total pages
- Zoom level
- Compilation status (compiling, success, error)
- Last compilation time
- Document dimensions

**Context Menu (Right-click):**

- Copy text selection (if text selectable)
- Copy as image
- Save page as...
- Zoom to selection
- Rotate page
- Print page
- Page properties

### 5.6 Performance Optimization

**Memory Management:**

**Page Texture Caching:**

- Cache rendered pages as GPU textures
- LRU eviction: keep most recently viewed pages
- Memory budget: configurable max cache size (default 200MB)
- Discard cache when document recompiled (pages invalidated)

**Lazy Loading:**

- Load pages on-demand as user scrolls
- Show placeholder while page rendering
- Priority queue: visible pages highest priority
- Background rendering of nearby pages

**Rendering Optimization:**

**Viewport Culling:**

- Only render pages intersecting viewport
- Calculate visible page range efficiently
- Update culled set on scroll/zoom

**Level of Detail:**

- Render lower quality when zoomed out significantly
- Render high quality when zoomed in
- Adaptive quality based on zoom level

**Incremental Updates:**

- When document recompiled, detect which pages changed
- Only re-render changed pages
- Reuse textures for unchanged pages

**GPU Utilization:**

- Offload rendering to GPU where possible
- Use GPU for scaling and transformations
- Batch render operations to reduce draw calls

**Compilation Performance:**

**Smart Recompilation:**

- Track edit locations and types
- For trivial edits (typo fixes), use fast path
- For structural changes (adding sections), use full path
- Cancel in-flight compilation if new edit arrives

**Background Compilation:**

- Compile in background thread to keep UI responsive
- Show progress indicator during compilation
- Allow cancellation of long compilations

**Compilation Timeouts:**

- Set maximum compilation time (default: 30s)
- If exceeded, show warning and allow user to:
  - Cancel and edit further
  - Wait longer
  - Force compilation with optimizations disabled

---

## Phase 6: Advanced Editor Features (Weeks 16-18)

### 6.1 Multi-Cursor and Multi-Selection

**Multi-Cursor Modes:**

**Adding Cursors:**

- Ctrl/Cmd+Click: add cursor at click position
- Ctrl/Cmd+D: select next occurrence of word under cursor, add cursor
- Ctrl/Cmd+Shift+L: add cursor to end of each selected line
- Alt+Drag: add cursors in rectangular column
- Ctrl/Cmd+Alt+Up/Down: add cursor above/below

**Cursor Management:**

- Track list of cursor positions (primary + secondaries)
- Each cursor has independent selection
- Cursors merge if they overlap after edit
- Escape or click removes all secondary cursors

**Editing with Multiple Cursors:**

- Type: insert at all cursors simultaneously
- Delete/Backspace: delete at all cursors
- Paste: paste same content at all cursors (or split by lines if counts match)
- Arrow keys: move all cursors together
- Home/End: move all cursors to line boundaries

**Visual Representation:**

- Primary cursor: solid color, blinking
- Secondary cursors: same color, slightly transparent, blinking in sync
- Each cursor's selection highlighted independently

### 6.2 Search and Replace

**Search Panel:**

**Basic Search:**

- Search input field with incremental search
- Results highlighted in editor as you type
- Result counter: "3 of 15"
- Navigate: Enter (next), Shift+Enter (previous)
- F3/Shift+F3 for next/previous
- Escape to close search

**Search Options:**

- Case sensitive toggle
- Whole word match toggle
- Regex mode toggle
- Search in selection only toggle

**Search Scope:**

- Current file (default)
- All open files
- Entire workspace (uses LSP or ripgrep)
- Specific folder

**Visual Indicators:**

- All matches highlighted in scrollbar
- Current match highlighted differently (brighter)
- Match count indicator
- Search term highlighted in results list

**Replace:**

**Replace Panel:**

- Replace input field
- Replace button (replace current match)
- Replace All button (replace all matches)
- Replace in selection (if selection active)

**Replace Preview:**

- Show diff of changes before applying
- Allow user to review and select which to replace
- Checkbox list of all matches with context
- Apply Selected button

**Regex Replace:**

- Capture groups: $1, $2, etc. in replacement
- Case transformation: \u, \U, \l, \L
- Conditional replacement based on capture groups

**Find in Files:**

**Search Interface:**

- Dedicated panel/tab for workspace search
- Search query with same options as editor search
- File pattern include/exclude (\*.typ, !tests/)
- Results tree grouped by file

**Results Display:**

- File path with match count
- Each match with line number and context (1 line before/after)
- Syntax highlighting in results
- Click to navigate to match
- Replace in Files feature (with preview)

### 6.3 Code Snippets

**Snippet System:**

**Snippet Definition:**

```
Snippet {
    trigger: String,              // Text to trigger snippet
    name: String,                 // Display name
    description: String,          // Tooltip description
    body: String,                 // Snippet template with placeholders
    scope: Option<String>,        // Language/context where available
    prefix_match: PrefixMatchMode // How to match trigger (exact, fuzzy)
}
```

**Snippet Syntax:**

- `$1`, `$2`, ...: Tab stops (cursor positions)
- `${1:placeholder}`: Tab stop with default text
- `${1|option1,option2|}`: Tab stop with choices
- `$0`: Final cursor position
- `$VARIABLE` or `${VARIABLE:default}`: Variables
  - `$TM_SELECTED_TEXT`: Currently selected text
  - `$TM_CURRENT_LINE`: Current line content
  - `$TM_FILENAME`: Current file name
  - `$TM_DIRECTORY`: Current directory
  - `$CLIPBOARD`: Clipboard content

**Snippet Expansion:**

- Trigger on Tab after typing snippet prefix
- Show snippet completion suggestions
- Preview snippet in completion popup
- Insert snippet and enter snippet mode

**Snippet Mode:**

- Highlight current tab stop
- Tab to move to next tab stop
- Shift+Tab to move to previous
- Type to replace placeholder
- Escape to exit snippet mode
- Enter on final tab stop ($0) exits mode

**Built-in Snippets for Typst:**

- Document template: `doc` → full document structure
- Function definition: `func` → function with parameters
- Heading levels: `h1`, `h2`, etc.
- Emphasis: `bold`, `italic`, `link`
- Lists: `ul` (unordered), `ol` (ordered)
- Math blocks: `math`, `align`
- Code blocks: `code`, `raw`
- Tables: `table`
- Figures: `fig`, `img`
- References: `ref`, `cite`

**Custom Snippets:**

- User-defined snippets in configuration file
- JSON or YAML format
- Global snippets vs. project-specific
- Import/export snippet collections
- Snippet editor UI (future enhancement)

### 6.4 Code Folding

**Fold Types:**

1. **Syntax-based folding**:

   - Function bodies
   - Block structures (#[...])
   - Conditional blocks (#if / #else)
   - Loop bodies
   - Comments (block and adjacent line comments)

2. **Indentation-based folding**:

   - Fallback when syntax unclear
   - Fold regions with consistent indentation
   - Useful for structured data

3. **Manual folding regions**:
   - Special comments: `// region: Name` ... `// endregion`
   - Allow arbitrary sections to be folded
   - Useful for organization

**Folding UI:**

**Gutter Indicators:**

- Triangle icon in gutter next to foldable lines
- ▼ when unfolded, ▶ when folded
- Hover shows tooltip: "Fold X lines"
- Click to toggle fold

**Folded Display:**

- Replace folded lines with single line
- Show fold summary: `{...}` or `[... 15 lines]`
- Different styling (italic, dimmed)
- Hover over fold shows preview popup of hidden content
- Click fold line to unfold

**Keyboard Shortcuts:**

- Ctrl/Cmd+K, Ctrl/Cmd+0: Fold all
- Ctrl/Cmd+K, Ctrl/Cmd+J: Unfold all
- Ctrl/Cmd+K, Ctrl/Cmd+L: Toggle fold at cursor
- Ctrl/Cmd+K, Ctrl/Cmd+1-9: Fold level N

**Fold Persistence:**

- Remember fold states per file
- Restore fold state when file reopened
- Per-workspace fold state storage

**Folding Behavior:**

- Folding respects selection: can't fold across selection
- Copy/paste: copied text includes folded content
- Search: search within folded regions, temporarily unfold to show match
- Syntax highlighting: maintain highlight state for folded content
- Edit in fold: automatically unfold if user tries to edit folded region

### 6.5 Split Panes and Layout

**Pane Management:**

**Split Operations:**

- Split horizontal: Divide editor vertically (side-by-side)
- Split vertical: Divide editor horizontally (top/bottom)
- Split with same file: Two views of same document (synchronized scrolling option)
- Split with different file: Independent documents
- Maximum splits: 4 panes recommended (configurable)

**Pane Layout:**

```
Workspace {
    root_pane: PaneGroup,
    active_pane: PaneId,
}

PaneGroup {
    split_direction: Direction (Horizontal/Vertical),
    children: Vec<Pane>,
    split_ratio: Vec<f32>,  // Proportions of each child
}

Pane {
    kind: PaneKind (Editor / Preview / Terminal / Other),
    content: Box<dyn PaneContent>,
}
```

**Pane Navigation:**

- Click to focus pane
- Keyboard: Ctrl+1-9 to focus pane by number
- Cycle: Ctrl+Tab to next pane, Ctrl+Shift+Tab to previous
- Direction: Ctrl+Alt+Arrow to move focus in that direction
- Swap panes: Drag pane tab to another pane

**Splitter Controls:**

- Draggable splitter between panes
- Double-click splitter to equalize sizes
- Keyboard: Ctrl+Alt+= to equalize all, Ctrl+Alt++ to grow, Ctrl+Alt+- to shrink
- Minimum pane size: 200px width, 100px height

**Pane Options:**

- Maximize pane (hide others temporarily)
- Close pane (merge with sibling)
- New pane (create editor in new split)
- Move pane to new window
- Swap pane positions

**Layout Presets:**

- 50/50 vertical split (editor | preview)
- 70/30 vertical split (large editor | narrow preview)
- Horizontal split (editor top | preview bottom)
- Three-column (outline | editor | preview)
- Save custom layout presets

### 6.6 File Management and Navigation

**File Explorer Sidebar:**

**Tree View:**

- Show project directory tree
- File and folder icons (by type)
- Expand/collapse folders
- Show hidden files toggle
- Sort options (name, type, modified date)
- Search/filter files in tree

**File Operations:**

- New file/folder
- Rename (F2)
- Delete (Delete key, with confirmation)
- Duplicate
- Move (drag and drop)
- Copy/paste files
- Show in system file manager

**Context Menu:**

- Open file
- Open in new window
- Open with default app
- Copy path / relative path
- Reveal in file explorer
- New file here
- New folder here
- Delete
- Rename

**File Status Indicators:**

- Modified (dot or star)
- Read-only (lock icon)
- Git status (M, A, D, U indicators)
- Ignored (dimmed)

**Quick Open:**

**File Picker (Ctrl/Cmd+P):**

- Fuzzy search across all project files
- Show recent files first
- Show file path and preview
- Navigate with arrows, Enter to open
- Ctrl+Enter to open in new split
- Search as you type with instant results

**Recent Files:**

- List of recently opened files
- Pinned files stay at top
- Clear recent history option

**Go to Symbol (Ctrl/Cmd+T):**

- Search for symbols across workspace
- Fuzzy match on symbol name
- Show file and line number
- Navigate to symbol on selection

**Breadcrumbs:**

**Path Breadcrumbs:**

- Show file path at top of editor
- Each segment clickable (shows siblings)
- Show current folder structure
- Navigate up folder hierarchy

**Symbol Breadcrumbs:**

- Show current symbol hierarchy
- Function → nested function → current position
- Click to jump to parent symbol
- Dropdown shows sibling symbols

---

## Phase 7: Polish and Optimization (Weeks 19-20)

### 7.1 Performance Profiling and Optimization

**Profiling Tools:**

**Built-in Profilers:**

- Frame time profiler: Track time per frame (target: <16ms for 60fps)
- Render pipeline profiler: Break down render stages
- Memory profiler: Track allocations, heap usage, leaks
- LSP profiler: Track request/response latencies
- Compilation profiler: Time each compilation stage

**Performance Metrics Dashboard:**

- Real-time FPS counter
- Memory usage graph
- CPU usage by thread
- GPU utilization
- LSP request queue length
- Compilation queue depth
- Cache hit rates

**Profiling Workflow:**

1. Enable profiling mode via dev menu
2. Perform actions to profile (editing, scrolling, etc.)
3. Export profiling data to JSON
4. Analyze with external tools (Chrome DevTools, Tracy)

**Critical Optimization Targets:**

**Text Rendering:**

- Target: <5ms to render visible text
- Optimize: Glyph caching, shaped text reuse
- Lazy shape: Only shape visible lines
- GPU upload: Batch glyph texture updates

**Scrolling:**

- Target: Locked 60fps during scroll
- Optimize: Virtual scrolling, render ahead
- Throttle: LSP updates during rapid scroll
- Cache: Rendered line textures

**Typing Latency:**

- Target: <50ms from keystroke to screen update
- Optimize: Fast path for character insertion
- Defer: Syntax highlighting, LSP notifications
- Prioritize: Cursor movement and character display

**Compilation:**

- Target: <500ms for typical document changes
- Optimize: Incremental compilation, caching
- Cancel: Abandon old compilations quickly
- Parallel: Compile imported files simultaneously

### 7.2 Memory Optimization

**Memory Budget Allocation:**

Total target: <500MB for typical session

- Text buffers: 50MB (including undo history)
- Rendered line cache: 100MB
- LSP data structures: 50MB
- Compilation cache: 100MB
- Preview textures: 150MB
- UI framework: 50MB

**Memory Reduction Strategies:**

**Text Buffer:**

- Use rope structure (minimal memory overhead)
- Compact undo history (merge consecutive edits)
- Discard old undo entries beyond limit
- Share immutable text between snapshots

**Render Cache:**

- LRU eviction for offscreen lines
- Compress cached textures when inactive
- Discard invisible line renders
- Reference counting for shared glyphs

**Preview:**

- Render only visible pages + buffer
- Use mipmap levels for zoomed-out views
- Compress page textures (GPU compression)
- Stream large PDFs page-by-page

**LSP Data:**

- Incremental symbol tables (not full copies)
- Weak references to rarely used data
- Periodic garbage collection of stale data
- Compact diagnostic storage

**Memory Leak Prevention:**

**Leak Detection:**

- Track allocations in development builds
- Periodic heap snapshots
- Compare snapshots to find leaks
- Automated leak tests in CI

**Common Leak Sources:**

- Circular references (use weak pointers)
- Event handlers not unsubscribed
- Cached data never evicted
- GPU resources not freed

**Mitigation:**

- RAII patterns (Rust ownership helps)
- Explicit cleanup in Drop implementations
- Validation: All resources freed on close
- Memory sanitizers in debug builds

### 7.3 Startup Time Optimization

**Startup Performance Targets:**

- Cold start: <2 seconds to window display
- Warm start: <1 second to usable editor
- Resume session: <500ms to restored state

**Startup Phases:**

**Phase 1: Application Launch (0-200ms):**

- Binary loading (OS responsibility)
- Initialize logging system
- Parse command line arguments
- Load critical configuration only
- Initialize GPUI runtime

**Phase 2: Window Creation (200-500ms):**

- Create main application window
- Initialize GPU context
- Load theme (cached if possible)
- Show splash screen or skeleton UI
- Begin async initialization

**Phase 3: Workspace Restoration (500-1500ms):**

- Load workspace state from disk
- Restore open files (lazy load content)
- Restore split pane layout
- Start LSP in background
- Show editor as soon as first file loaded

**Phase 4: Full Initialization (1500-2000ms):**

- Complete LSP initialization
- Index project files (in background)
- Load plugins/extensions
- Restore scroll positions and selections
- Show fully functional editor

**Lazy Loading Strategy:**

- Defer non-essential initialization
- Load fonts on-demand (first use)
- Index files in background (low priority)
- Load syntax highlighting grammars lazily
- Cache parsed configuration

**Session Restoration:**

**State to Restore:**

- Open files and dirty buffers
- Cursor positions and selections
- Scroll positions in each editor
- Split pane layout
- Sidebar visibility and selections
- Recent search queries
- Undo history (optional, can be large)

**Fast Restoration:**

- Store session state in binary format (not JSON)
- Incremental writes during session
- Read only what's needed for display
- Lazy load full undo history
- Background validation of restored state

### 7.4 User Experience Polish

**Visual Refinements:**

**Animations:**

- Smooth cursor movement (ease-out)
- Smooth scrolling (momentum and easing)
- Fade in/out for tooltips and popups
- Slide animations for panels
- Highlight pulse for sync/search results
- Loading spinners for async operations

**Timing:**

- All animations <300ms duration
- No animation blocking user input
- Instant feedback for all interactions
- Cancellable animations (new action stops old)

**Transitions:**

- Theme changes: smooth color transitions
- Pane resizing: smooth size adjustments
- Tab switching: slide or fade effect
- Panel show/hide: slide with easing

**Visual Feedback:**

**Interactive States:**

- Hover: Highlight buttons, links, gutter items
- Active: Darken/lighten pressed buttons
- Focus: Border or background change for focused elements
- Disabled: Dim and de-saturate disabled items
- Loading: Spinner or progress indicator

**Progress Indication:**

- Compilation: Progress bar in status bar
- File operations: Modal progress dialog for slow ops
- Background tasks: Unobtrusive indicator in corner
- Cancellable: Show cancel button for long operations

**Error States:**

- Validation errors: Red border, inline message
- Missing files: Yellow background, warning icon
- LSP disconnected: Status bar indicator
- Compilation failed: Red icon with error count

**Accessibility:**

**Keyboard Navigation:**

- Full keyboard control (no mouse required)
- Tab navigation through all UI elements
- Escape to close popups/dialogs
- Shortcuts discoverable in menus/tooltips
- Customizable keyboard shortcuts

**Screen Reader Support:**

- ARIA labels for all UI elements
- Announce editor changes to screen readers
- Describe cursor position and selections
- Read diagnostic messages
- Announce compilation status

**Visual Accessibility:**

- High contrast mode
- Adjustable font sizes (UI and editor separate)
- Colorblind-friendly themes
- Cursor width and blink rate adjustable
- Line height adjustable

**Motor Accessibility:**

- Click target sizes: minimum 32x32 pixels
- Drag distance threshold configurable
- Double-click timing adjustable
- Sticky keys support
- Mouse keys support

**Sound Feedback (Optional):**

- Audible bell for errors
- Sound on compilation complete
- Mute option for all sounds
- Different sounds for different event types

### 7.5 Theme System

**Theme Structure:**

```
Theme {
    name: String,
    variant: Variant (Light/Dark),
    colors: ColorScheme {
        // UI colors
        background: Color,
        foreground: Color,
        border: Color,
        selection: Color,
        cursor: Color,
        current_line: Color,

        // Syntax colors
        keyword: Color,
        function: Color,
        variable: Color,
        constant: Color,
        string: Color,
        comment: Color,
        type_name: Color,
        operator: Color,

        // Semantic colors
        error: Color,
        warning: Color,
        info: Color,
        hint: Color,

        // UI element colors
        button_background: Color,
        button_hover: Color,
        input_background: Color,
        panel_background: Color,
        sidebar_background: Color,
        statusbar_background: Color,
    },
    typography: Typography {
        editor_font: FontFamily,
        editor_size: f32,
        ui_font: FontFamily,
        ui_size: f32,
        line_height: f32,
    },
    spacing: Spacing {
        gutter_width: f32,
        line_padding: f32,
        panel_padding: f32,
    },
}
```

**Built-in Themes:**

- Light: Bright background, dark text (default light)
- Dark: Dark background, light text (default dark)
- High Contrast Light: Maximum contrast for accessibility
- High Contrast Dark: Dark version with high contrast
- Additional themes: Solarized, Monokai, Dracula, Nord, etc.

**Theme Customization:**

**User Overrides:**

- Override specific colors without full theme
- Per-workspace theme overrides
- Time-based theme switching (auto dark mode at night)
- Sync with system theme preference

**Theme Editor:**

- Visual theme editor (future enhancement)
- Live preview while editing
- Export theme to file for sharing
- Import themes from file or URL

**Theme Migration:**

- Detect theme format changes
- Migrate old theme files to new format
- Warn if theme missing required colors
- Fallback to defaults for missing values

### 7.6 Configuration System

**Configuration Hierarchy:**

1. **Default Configuration**: Built-in defaults
2. **System Configuration**: OS-level settings (/etc or registry)
3. **User Configuration**: ~/.config/typst-editor/config.toml
4. **Workspace Configuration**: .typst-editor/config.toml in project root
5. **Document Configuration**: Frontmatter or file-specific settings

Each level overrides previous, allowing granular control.

**Configuration Format:**

Use TOML for human readability:

```toml
[editor]
font_family = "JetBrains Mono"
font_size = 14.0
line_height = 1.5
tab_size = 2
insert_spaces = true
word_wrap = false
show_line_numbers = true

[editor.cursor]
style = "block"  # block, line, underline
blink_rate = 500

[bidi]
auto_detect_direction = true
default_direction = "ltr"
show_direction_marks = true

[lsp]
enable = true
diagnostic_delay = 500
completion_trigger_delay = 100

[preview]
default_zoom = "fit_width"
sync_scroll = true
render_quality = "normal"  # draft, normal, high

[compilation]
debounce_delay = 300
incremental = true
max_compile_time = 30000

[keybindings]
"ctrl+s" = "save"
"ctrl+shift+f" = "format_document"
# ... more keybindings
```

**Configuration Validation:**

- Schema validation on load
- Type checking for all values
- Range validation (e.g., font size 6-72)
- Enum validation for choice fields
- Warn about unknown keys (typos)
- Error reporting with line numbers

**Configuration UI:**

**Settings Panel:**

- Categorized settings tree
- Search settings by name/description
- Show current value and default
- Input widgets appropriate for type:
  - Text fields for strings
  - Number spinners for integers/floats
  - Checkboxes for booleans
  - Dropdowns for enums
  - Color pickers for colors
- Reset to default button per setting
- Show which config level sets each value

**Hot Reload:**

- Watch config files for changes
- Reload and apply changes immediately
- No restart required for most settings
- Validate before applying (rollback if invalid)
- Show notification when config reloaded

### 7.7 Error Handling and Diagnostics

**Error Categories:**

1. **User Errors** (expected, common):

   - File not found
   - Syntax errors in Typst code
   - Invalid configuration
   - File permission issues

2. **Application Errors** (unexpected, need graceful handling):

   - LSP crash or disconnect
   - Compilation timeout or failure
   - Out of memory
   - GPU errors

3. **Critical Errors** (rare, may require restart):
   - Corrupted workspace state
   - GPUI framework panic
   - Filesystem errors
   - System resource exhaustion

**Error Presentation:**

**User Error Display:**

- Inline in editor (squiggly underlines)
- Diagnostic panel with full details
- Toast notification for file operations
- Modal dialog only when blocking action needed

**Application Error Display:**

- Status bar indicator (yellow/red)
- Detailed error in panel (expandable)
- Offer recovery actions when possible
- Log to file for debugging

**Critical Error Display:**

- Modal dialog explaining issue
- Options: Restart, Safe Mode, Report Issue
- Attempt to save unsaved work
- Show log file location

**Error Recovery:**

**Automatic Recovery:**

- Restart crashed LSP automatically
- Retry failed file operations (with backoff)
- Reload corrupted cache from backup
- Rebuild indexes if corrupted

**User-Initiated Recovery:**

- "Retry" button for failed operations
- "Reset to defaults" for config errors
- "Safe mode" startup bypasses extensions
- "Clear cache" to resolve cache issues

**Error Reporting:**

**Crash Reports:**

- Collect crash information (stack trace, logs)
- Ask user permission to send report
- Include system info (OS, version, etc.)
- Exclude sensitive data (file contents)
- Send to telemetry endpoint (opt-in)

**Bug Report Template:**

- Pre-filled with version, OS, error details
- Link to open issue in GitHub
- Attach relevant logs
- Steps to reproduce if known

### 7.8 Testing Strategy

**Unit Tests:**

**Core Components:**

- Text buffer operations (insert, delete, undo)
- Bidirectional text algorithm
- Cursor movement logic
- Selection handling
- Configuration parsing
- Keyboard shortcut matching

**Test Coverage Target:** >80% for core crates

**Integration Tests:**

**Editor Workflows:**

- Open file, edit, save, close
- Multi-cursor editing sequences
- Search and replace operations
- Undo/redo chains
- LSP request/response cycles
- Compilation pipeline end-to-end

**Test Approach:**

- Simulated user input
- Verify state after operations
- Check UI updates (via GPUI test utilities)
- Mock LSP responses for determinism

**UI Tests:**

**Component Tests:**

- Render components in test environment
- Verify layout calculations
- Check event handling
- Validate accessibility properties

**Visual Regression Tests:**

- Capture screenshots of UI components
- Compare against baseline images
- Detect unintended visual changes
- Regenerate baselines when changes intentional

**Performance Tests:**

**Benchmarks:**

- Buffer operations (insert, delete) on large files
- Syntax highlighting speed
- Compilation time for various document sizes
- Preview rendering speed
- Scrolling frame rate

**Continuous Monitoring:**

- Track performance over commits
- Alert if performance regresses >10%
- Benchmark suite in CI
- Historical performance graphs

**Platform Tests:**

**Cross-Platform Validation:**

- Automated tests on macOS, Linux, Windows
- Test platform-specific code paths
- Verify keyboard shortcuts work on all platforms
- Check font rendering consistency

**Manual Testing Checklist:**

- Hardware acceleration (dedicated GPU)
- HiDPI display support
- Dark mode appearance
- Window management (minimize, maximize, full-screen)
- File associations (open .typ files)

**Typst-Specific Tests:**

**Document Tests:**

- Sample Typst documents of varying complexity
- Multilingual documents (RTL, LTR, CJK)
- Large documents (>10,000 lines)
- Documents with many imports
- Documents with complex math

**Edge Cases:**

- Empty document
- Very long lines (>10,000 characters)
- Deeply nested structures
- Unicode edge cases (emoji, combining characters)
- Invalid Typst syntax

### 7.9 Documentation

**User Documentation:**

**Getting Started Guide:**

- Installation instructions per platform
- First-time setup wizard walkthrough
- Creating first Typst document
- Basic editing tutorial
- Preview and export tutorial

**User Manual:**

- Complete feature documentation
- Keyboard shortcuts reference
- Configuration reference
- Troubleshooting guide
- FAQ

**Video Tutorials:**

- Editor overview (5 minutes)
- Advanced editing features (10 minutes)
- Working with bidirectional text (5 minutes)
- Project management (7 minutes)

**Developer Documentation:**

**Architecture Overview:**

- System design document
- Component interaction diagrams
- Threading model explanation
- State management approach

**API Documentation:**

- Rustdoc for all public APIs
- Code examples for key components
- Extension/plugin API (if applicable)

**Contribution Guide:**

- Setting up development environment
- Building and running tests
- Code style guidelines
- Pull request process
- Issue reporting guidelines

**LSP Integration Guide:**

- How to debug LSP issues
- LSP message logging
- Custom LSP settings
- Working with Typst LSP source

---

## Phase 8: Release and Deployment (Week 21+)

### 8.1 Alpha Release (Week 21)

**Alpha Features:**

- Core editing with bidirectional text
- Basic Typst compilation
- Simple preview
- Essential LSP features (diagnostics, completion)
- Minimal UI polish

**Alpha Goals:**

- Internal testing
- Gather early feedback on architecture
- Identify critical bugs
- Performance baseline

**Alpha Distribution:**

- GitHub releases page
- Pre-compiled binaries for major platforms
- Build-from-source instructions
- Known issues documented

### 8.2 Beta Release (Week 24)

**Beta Features:**

- All planned features implemented
- LSP fully integrated
- Preview with source sync
- Multi-cursor, search, snippets
- Configuration system
- Theme support

**Beta Goals:**

- Public testing by early adopters
- Real-world usage feedback
- Bug fixing and stabilization
- Performance optimization
- Documentation completion

**Beta Distribution:**

- Public announcement (Reddit, HN, forums)
- Auto-update mechanism (optional)
- Feedback channels (Discord, GitHub Discussions)
- Beta user community

### 8.3 Release Candidate (Week 26)

**RC Criteria:**

- No known critical bugs
- Performance targets met
- Documentation complete
- Accessibility standards met
- Security audit passed

**RC Testing:**

- Extended testing period (2 weeks)
- Bug bounty program
- Stress testing
- Compatibility testing
- Final performance profiling

### 8.4 Version 1.0 Release (Week 28)

**Release Readiness:**

- All RC issues resolved
- Release notes finalized
- Marketing materials prepared
- Support channels ready
- Telemetry and crash reporting operational

**Launch Activities:**

- Press release / blog post
- Social media announcement
- Product Hunt / HN launch
- Community engagement
- Monitor for launch day issues

**Post-Launch:**

- Hotfix readiness (24/7 on-call first week)
- User feedback monitoring
- Rapid bug fixes for critical issues
- Plan next version features

### 8.5 Continuous Delivery

**Release Cadence:**

- Major versions: Every 6 months (new features)
- Minor versions: Monthly (enhancements, non-breaking changes)
- Patch versions: As needed (bug fixes, security)

**Update Mechanism:**

**Auto-Update:**

- Check for updates on startup (daily)
- Download in background
- Notify user when ready
- Install on restart
- Rollback option if issues

**Update Channels:**

- Stable: Tested releases only
- Beta: Early access to new features
- Nightly: Latest development builds
- User chooses channel in settings

**Release Process:**

1. **Development**: Feature branches merged to main
2. **CI/CD**: Automated builds and tests
3. **Staging**: Deploy to beta channel
4. **Testing**: Beta users test for 1-2 weeks
5. **Release**: Promote to stable channel
6. **Monitor**: Watch metrics and crash reports
7. **Hotfix**: Address critical issues immediately

---

## Appendices

### Appendix A: Technology Deep Dives

**GPUI Framework Details:**

GPUI is Zed's UI framework, designed for performance:

- GPU-accelerated rendering
- Reactive state management
- Platform abstractions (macOS, Linux, Windows)
- Text rendering with complex script support
- Accessibility built-in

**Integration Considerations:**

- Use GPUI's text rendering for consistency
- Leverage reactive components for UI updates
- Platform-specific code via GPUI abstractions
- Test using GPUI's test utilities

**Bidirectional Text Algorithm (UAX #9):**

Unicode Standard Annex #9 defines the algorithm:

1. **Explicit embeddings**: Process RLE, LRE, RLO, LRO, PDF, RLI, LRI, FSI, PDI
2. **Resolve weak types**: Numbers, European number separators, etc.
3. **Resolve neutral types**: Spaces, punctuation based on context
4. **Resolve implicit levels**: Based on surrounding characters
5. **Reorder for display**: Create visual ordering from logical
6. **Mirror glyphs**: Flip brackets and other paired characters

**Implementation:** Use `unicode-bidi` crate which fully implements UAX #9.

**Typst Compiler Integration:**

Typst provides:

- `typst` crate: Core compiler
- `typst-syntax`: Parser and syntax tree
- `typst-library`: Standard library
- `typst-pdf`: PDF export
- `typst-svg`: SVG export

**World Trait:**
Must implement `typst::World`:

- `library()`: Provide standard library
- `book()`: Provide font catalog
- `main()`: Provide main source file
- `source(id)`: Load source by ID
- `file(id)`: Load binary file
- `font(index)`: Load font by index
- `today()`: Provide current date

### Appendix B: Performance Targets

**Latency Targets:**

- Keystroke to display: <50ms (perceivably instant)
- Hover to tooltip: <300ms (comfortable delay)
- Completion trigger to popup: <100ms (responsive)
- Compilation to preview: <500ms (typical edit)
- Search in file: <100ms (feels instant)
- Open file: <200ms (acceptable)
- Startup: <2s (tolerable)

**Throughput Targets:**

- Scrolling: 60 FPS (smooth)
- Typing: Handle 200 WPM (very fast typist)
- File operations: 100+ files/second
- Search: 1 GB/second (using ripgrep)

**Scalability Targets:**

- File size: 100,000 lines (large document)
- Project size: 10,000 files (large codebase)
- Undo history: 1,000 operations (reasonable memory)
- Open editors: 20 simultaneous (many tabs)

### Appendix C: Security Considerations

**Code Execution:**

- LSP runs in separate process (sandboxed)
- Typst compilation in separate process
- No eval or dynamic code execution from documents
- File access limited to project root

**File System:**

- Path traversal prevention
- Symlink attack prevention
- Temp file cleanup
- Secure file permissions

**Network:**

- HTTPS only for package downloads
- Certificate validation
- Timeout on network requests
- User consent for network access

**Privacy:**

- No telemetry without opt-in
- Crash reports sanitized
- Local processing (no cloud requirement)
- Configuration encrypted if contains secrets

**Updates:**

- Signed binaries (code signing)
- Checksum verification
- Secure update channel (HTTPS)
- Rollback capability

### Appendix D: Accessibility Standards

**WCAG 2.1 Level AA Compliance:**

**Perceivable:**

- Color contrast: 4.5:1 for normal text, 3:1 for large
- Zoom: UI scales to 200% without loss of functionality
- Keyboard navigation: All features accessible via keyboard
- Screen reader: ARIA labels and announcements

**Operable:**

- Keyboard shortcuts: No timing requirements
- Focus visible: Clear focus indicators
- Skip navigation: Jump to main content
- Motion: Disable animations option

**Understandable:**

- Language: Consistent terminology
- Error messages: Clear and actionable
- Help: Context-sensitive help available

**Robust:**

- Standards: Use semantic HTML/accessibility APIs
- Compatibility: Test with multiple screen readers

### Appendix E: Localization Plan

**Internationalization (i18n):**

- UI strings externalized to resource files
- Date/time formatting: Locale-aware
- Number formatting: Locale-aware
- Keyboard layouts: Platform-appropriate

**Translation (l10n):**

**Priority Languages:**

1. English (default, US and UK)
2. Spanish
3. French
4. German
5. Chinese (Simplified and Traditional)
6. Arabic (RTL testing)
7. Japanese
8. Russian

**Translation Infrastructure:**

- Use `fluent` crate for translations
- Translation files in FTL format
- Crowdsourced translations (Crowdin, POEditor)
- Regular translation updates

**RTL Language Support:**

- UI mirrors for RTL languages
- Text direction auto-detection
- Mixed directionality in UI
- Thorough testing with Arabic/Hebrew

---

## Conclusion

This specification provides a comprehensive roadmap for building a sophisticated Typst editor with bidirectional text support. Key success factors:

1. **Strong Architecture**: Solid foundation enables feature addition
2. **Performance Focus**: Optimize from the start, don't defer
3. **User Experience**: Smooth, polished interactions throughout
4. **LSP Integration**: Leverage Typst LSP for intelligence
5. **Testing**: Comprehensive testing prevents regressions
6. **Documentation**: Help users and developers understand the system

**Next Steps:**

1. Review and refine specification
2. Set up development environment
3. Begin Phase 1 implementation
4. Establish weekly progress reviews
5. Build and maintain momentum

**Success Metrics:**

- Editor startup time <2s
- Keystroke latency <50ms
- Compilation time <500ms for typical edits
- User satisfaction score >4.5/5
- Crash rate <0.1% of sessions
