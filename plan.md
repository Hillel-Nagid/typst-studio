# Typst Studio - Detailed Specification

## Native Cross-Platform Typst Editor

## Project Overview

A high-performance native text editor for Typst documents with bidirectional text support (RTL/LTR), live preview capabilities, and full LSP integration, built using GPUI framework.

## Technology Stack

- **UI Framework**: GPUI (Rust-based GPU-accelerated UI)
- **Language**: Rust (Edition 2021)
- **Text Processing**: Typst compiler + Typst LSP
- **Text Rendering**: Unicode bidirectional algorithm (UAX #9) via `unicode-bidi` crate
- **Text Buffer**: Rope data structure via `ropey` crate
- **PDF Preview**: Native rendering using `pdfium-render` (PDFium bindings)
- **LSP**: Typst's built-in language server
- **Async Runtime**: Tokio for async operations
- **Serialization**: Serde for JSON/TOML configuration
- **File Watching**: `notify` crate for filesystem monitoring

---

## Phase 1: Foundation & Architecture

### 1.1 Project Setup & Build Infrastructure

**Repository Structure:**

```
typst-studio/
├── crates/
│   ├── editor_core/           # Core editing logic
│   │   ├── buffer/            # Text buffer implementation
│   │   ├── selection/         # Selection and cursor management
│   │   ├── bidi_text/         # Bidirectional text handling
│   │   │   ├── algorithm/     # UAX #9 implementation
│   │   │   ├── layout/        # Visual layout engine
│   │   └── operations/        # Edit operations (insert, delete, etc.)
│   ├── typst_integration/     # Typst compiler wrapper
│   │   ├── compiler/          # Compilation service
│   │   ├── diagnostics/       # Error handling
│   │   ├── lsp_client/        # LSP client implementation
│   │   │   ├── protocol/      # LSP protocol handlers
│   │   │   ├── requests/      # Request management
│   │   │   └── notifications/ # Notification handling
│   │   └── world/             # File system abstraction
│   ├── preview/               # Preview rendering
│   │   ├── renderer/          # PDF/SVG rendering
│   │   ├── sync/              # Source-preview synchronization
│   │   └── viewport/          # Viewport management
│   └── ui/                    # UI components
│       ├── components/        # Reusable UI components
│       ├── workspace/         # Workspace UI and window UI
│       ├── navbar/            # Top Navigation bar
│       ├── editor/            # Main editor component
│       ├── preview/           # Preview component
│       ├── sidebar/           # File explorer, outline
│       └── panels/            # Building blocks of a window
│           ├── editor/        # The main editor panel
│           ├── preview/       # Preview panel
│           ├── sidebar/       # Sidebar panel
│           └── console/       # Console panel
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

**Development Environment:**

- Hot reload for UI development using GPUI's development features
- Logging infrastructure with multiple verbosity levels
- Debug overlays showing buffer state, bidi levels, LSP status
- Performance profiling integration with Tracy or similar tools
- Memory leak detection and sanitizers

**Cargo Workspace Structure:**

The project uses a Cargo workspace to organize multiple crates:

```toml
[workspace]
members = [
    "crates/editor_core",
    "crates/typst_integration",
    "crates/preview",
    "crates/ui",
]
resolver = "2"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# UI Framework
gpui = { git = "https://github.com/zed-industries/zed" }

# Async runtime
tokio = { version = "1.35", features = ["full"] }
futures = "0.3"

# Text processing
ropey = "1.6"
unicode-segmentation = "1.10"
unicode-bidi = "0.3"

# Typst integration
typst = "0.10"
typst-syntax = "0.10"

# PDF rendering
pdfium-render = "0.8"

# LSP
lsp-types = "0.95"
serde_json = "1.0"
serde = { version = "1.0", features = ["derive"] }

# Configuration
toml = "0.8"
directories = "5.0"

# File watching
notify = "6.1"

# Logging
tracing = "0.1"
tracing-subscriber = "0.3"

# Error handling
thiserror = "1.0"
anyhow = "1.0"

# Utilities
parking_lot = "0.12"
lru = "0.12"
regex = "1.10"
```

**Key Dependencies:**

1. **GPUI**: The core UI framework, provides GPU-accelerated rendering and reactive state management
2. **Ropey**: Efficient rope-based text buffer for handling large documents
3. **Unicode-bidi**: Implementation of the Unicode Bidirectional Algorithm (UAX #9)
4. **Typst**: The Typst compiler library for document compilation
5. **Pdfium-render**: Native PDF rendering using Google's PDFium library
6. **Tokio**: Async runtime for non-blocking operations (LSP, compilation, file I/O)
7. **LSP-types**: LSP protocol type definitions for language server communication
8. **Notify**: Cross-platform filesystem notification library
9. **typst-syntax** A typst code parser

**Platform-Specific Dependencies:**

```toml
[target.'cfg(target_os = "windows")'.dependencies]
windows = { version = "0.52", features = ["Win32_UI_Shell", "Win32_System_Com"] }

[target.'cfg(target_os = "macos")'.dependencies]
cocoa = "0.25"
objc = "0.2"

[target.'cfg(target_os = "linux")'.dependencies]
x11 = { version = "2.21", features = ["xlib"] }
```

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
   - Panels visibility

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

### 1.3 Data Models & State Architecture

**Document Model Specification:**

Each open file in the editor requires comprehensive metadata tracking:

- **Identity & Location:**

  - Unique document identifier (UUID-based) for tracking across application
  - File path (optional for unsaved documents)
  - Language type (Typst, with future extensibility)

- **Encoding & Format:**

  - Character encoding (UTF-8 primary, with UTF-16 and BOM variant support)
  - Line ending style (LF, CRLF, CR) with detection and preservation
  - Platform-aware defaults (CRLF on Windows, LF elsewhere)

- **State Tracking:**
  - Dirty flag for unsaved changes
  - Read-only status for protected files
  - Version counter for LSP synchronization (incremented on each change)
  - Last modified timestamp for conflict detection

**Project Model Specification:**

Projects represent collections of related Typst files with shared configuration:

- **Project Structure:**

  - Root directory defining project scope
  - Main file designation for compilation entry point
  - Dependency graph of imported/included files and assets

- **Dependency Tracking:**

  - File dependencies categorized by type:
    - Typst imports (`#import`)
    - Typst includes (`#include`)
    - Asset files (images, data files)
    - Typst packages (external dependencies)
  - Modification time tracking for cache invalidation
  - Automatic dependency discovery during compilation

- **Project Settings:**
  - Format-on-save configuration
  - Auto-compilation preferences
  - Custom compiler arguments
  - Output directory specification
  - Settings override capability per project

**Configuration Architecture:**

Hierarchical configuration system with four levels:

1. **Default Configuration:**

   - Hardcoded sensible defaults
   - Ensures application works out-of-box

2. **Global Configuration:**

   - User-wide settings stored in platform-specific locations:
     - Windows: `%APPDATA%\typst-studio\config.toml`
     - macOS: `~/Library/Application Support/typst-studio/config.toml`
     - Linux: `~/.config/typst-studio/config.toml`

3. **Workspace Configuration:**

   - Project-specific settings in `.typst-studio/config.toml`
   - Overrides global settings for specific workspace

4. **Document Configuration:**
   - Per-file overrides (future feature)
   - Highest priority

**Configuration Categories:**

- **Editor Settings:**

  - Typography: Font family, size, line height
  - Indentation: Tab size, spaces vs tabs
  - Display: Word wrap, line numbers, minimap visibility
  - Cursor: Style (block/line/underline), blink behavior
  - Auto-features: Auto-save, bracket/quote pairing

- **Appearance Settings:**

  - Theme selection (light/dark/high-contrast)
  - UI scale factor for accessibility
  - Sidebar position (left/right)
  - Icon theme

- **LSP Settings:**

  - Enable/disable language server
  - Server executable path
  - Completion trigger characters (e.g., `#`, `.`, `:`)
  - Hover delay (milliseconds)
  - Feature toggles (completion, hover, diagnostics, etc.)

- **Compiler Settings:**

  - Auto-compile triggers (on save, on change, manual only)
  - Compilation delay (debounce period in milliseconds)
  - Output verbosity
  - Custom compiler flags

- **Bidirectional Text Settings:**

  - Enable/disable bidi support
  - RTL line alignment behavior
  - Math mode detection for direction overrides

- **Keybindings:**
  - Customizable keyboard shortcuts
  - Platform-aware defaults (Cmd on macOS, Ctrl elsewhere)
  - Chord support (multi-key sequences)

**Configuration Features:**

- **Validation:**

  - Type checking and range validation
  - Immediate feedback on invalid values
  - Fallback to defaults on validation failure

- **Hot Reloading:**

  - File system watching for external changes
  - Automatic reload and re-merge on modification
  - No restart required for most settings

- **Import/Export:**
  - Configuration profile export for sharing
  - Profile import for quick setup
  - Version compatibility checking

**State Management Architecture:**

The application maintains state at multiple levels using GPUI's reactive patterns:

1. **Application-Level State:**

   - Window collection and active window tracking
   - Global configuration and theme
   - Recent files history
   - Workspace-to-window mapping

2. **Workspace-Level State:**

   - Workspace root directory
   - Open document collection
   - Active document tracking
   - Panel layout configuration (splits, visibility)
   - Sidebar, preview, and console visibility state

3. **Editor-Level State:**

   - Document metadata and content buffer
   - Multi-cursor positions and selections
   - Scroll position and viewport bounds
   - Undo/redo history
   - LSP diagnostics collection
   - Folded code regions
   - Syntax highlighting cache

4. **Preview-Level State:**
   - Compiled PDF data
   - Page count and current page
   - Zoom mode and level
   - Scroll position
   - Compilation status (idle, compiling, success, failed)
   - Source-preview sync markers

**Thread Safety Strategy:**

All shared state wrapped in `Arc<RwLock<T>>` for thread-safe access:

- UI thread: Read-heavy operations
- Background threads: Write operations for LSP, compilation, file I/O
- Lock contention minimization through fine-grained locking
- Deadlock prevention through consistent lock ordering

**State Synchronization Patterns:**

- **Event Bus:** Loosely coupled component communication
- **Reactive Subscriptions:** Components subscribe to state changes
- **Message Passing:** Async operations communicate via channels (tokio::mpsc)
- **Immutable Messages:** State changes communicated through immutable event types

---

## Phase 2: UI Foundation & Components

### 2.1 GPUI Setup & Window Management

**Core Window System:**

The foundation of the application starts with proper GPUI initialization and window management. This system must handle multiple windows, platform-specific considerations, and provide a solid base for all UI components.

**Implementation Requirements:**

- Initialize GPUI application with main event loop
- Implement `WindowManager` to handle multiple independent windows
- Create `MainWindow` component with proper DPI awareness for all platforms (Windows, macOS, Linux)
- Set up window state persistence (size, position, maximized state, split configuration)
- Implement platform-appropriate window titlebar:
  - Native titlebar on macOS with traffic lights
  - Custom titlebar on Windows and Linux with minimize/maximize/close buttons
- Handle window lifecycle events (create, focus, blur, close, minimize, maximize)
- Support full-screen mode with proper exit handling

**Key Files:**

- `src/main.rs` - Application entry point and GPUI initialization
- `crates/ui/workspace/window_manager.rs` - Window lifecycle management
- `crates/ui/workspace/main_window.rs` - Main window component
- `crates/ui/workspace/titlebar.rs` - Custom titlebar (Windows/Linux)

### 2.2 Theme System Implementation

**Theme Architecture:**

A comprehensive theming system is essential for both aesthetics and accessibility. The system must support multiple themes, smooth transitions, and hot-reloading for development.

**Theme Structure:**

```rust
pub struct Theme {
    // Background colors
    pub editor_background: Color,
    pub sidebar_background: Color,
    pub preview_background: Color,
    pub titlebar_background: Color,
    pub panel_background: Color,
    pub gutter_background: Color,

    // Foreground colors
    pub editor_foreground: Color,
    pub sidebar_foreground: Color,
    pub icon_color: Color,
    pub border_color: Color,
    pub divider_color: Color,

    // Semantic colors
    pub error: Color,
    pub warning: Color,
    pub info: Color,
    pub success: Color,
    pub hint: Color,

    // UI element colors
    pub selection_background: Color,
    pub selection_foreground: Color,
    pub cursor_color: Color,
    pub line_highlight: Color,
    pub matching_bracket: Color,
    pub button_background: Color,
    pub button_hover: Color,
    pub button_active: Color,
    pub input_background: Color,
    pub input_border: Color,

    // Syntax highlighting colors
    pub syntax: SyntaxColors,
}

pub struct SyntaxColors {
    pub keyword: Color,
    pub function: Color,
    pub variable: Color,
    pub string: Color,
    pub number: Color,
    pub comment: Color,
    pub type_name: Color,
    pub operator: Color,
    pub punctuation: Color,
    pub heading: Color,
    pub emphasis: Color,
    pub strong: Color,
    pub link: Color,
    pub code: Color,
}
```

**Features:**

- Support both light and dark themes with smooth transitions
- Include at least 4 built-in themes (Light, Dark, High Contrast Light, High Contrast Dark)
- Syntax highlighting color schemes optimized for Typst
- Theme hot-reloading for development (watch theme files for changes)
- Theme selection UI in settings with live preview
- Import/export custom themes
- Support for user-defined theme files (JSON/TOML format)

**Key Files:**

- `crates/ui/theme/mod.rs` - Theme definitions and management
- `crates/ui/theme/colors.rs` - Color palette definitions
- `crates/ui/theme/syntax.rs` - Syntax highlighting color schemes
- `crates/ui/theme/loader.rs` - Theme file loading and hot-reload
- `assets/themes/light.json` - Built-in light theme
- `assets/themes/dark.json` - Built-in dark theme
- `assets/themes/high_contrast_light.json` - High contrast light theme
- `assets/themes/high_contrast_dark.json` - High contrast dark theme

### 2.3 Core UI Components

**Reusable Component Library:**

Build a comprehensive set of reusable UI components that follow consistent design patterns and theming. All components must be keyboard-accessible and properly handle focus management.

**Component Specifications:**

1. **Button Component:**

   - Variants: Primary, Secondary, Danger, Ghost
   - States: Normal, Hover, Active, Disabled, Focus
   - Support for icons (left/right position)
   - Support for keyboard activation (Enter/Space)

2. **Input Component:**

   - Text input with validation support
   - Placeholder text with theme integration
   - Error state with validation message
   - Support for input masking (passwords, etc.)
   - Clear button (X) when not empty
   - Focus ring for accessibility

3. **Dropdown Component:**

   - Select menu with custom styling
   - Keyboard navigation (Arrow keys, Home, End, PageUp/Down)
   - Search/filter functionality
   - Support for grouped options
   - Custom option rendering (icons, descriptions)

4. **Tabs Component:**

   - Tab bar for managing multiple open files
   - Close buttons with confirmation for unsaved files
   - Drag-to-reorder tabs
   - Context menu (close, close others, close to the right)
   - Tab overflow handling (scroll or dropdown)
   - Dirty indicator (dot or asterisk for unsaved files)

5. **Splitter Component:**

   - Resizable split panes (horizontal/vertical)
   - Drag handle with hover feedback
   - Minimum/maximum size constraints
   - Double-click to reset to default size
   - Nested splitter support for complex layouts

6. **Scrollbar Component:**

   - Custom scrollbar matching theme
   - Smooth scrolling with momentum
   - Hover to show, auto-hide when inactive
   - Draggable thumb
   - Page up/down on track click
   - Keyboard support (arrow keys, PageUp/Down, Home/End)

7. **Context Menu Component:**

   - Right-click context menu
   - Keyboard navigation
   - Support for submenus
   - Icons and keyboard shortcuts display
   - Separator items
   - Disabled items
   - Position calculation to stay within window bounds

8. **Tooltip Component:**

   - Hover tooltips with configurable delay
   - Automatic positioning to avoid window edges
   - Support for rich content (not just text)
   - Keyboard-triggered tooltips (on focus)
   - Max width with text wrapping

9. **Icon Component:**

   - SVG icon rendering with theme color integration
   - Icon library with common icons (file types, actions, etc.)
   - Size variants (small, medium, large)
   - Support for custom SVG icons

10. **StatusBar Component:**
    - Bottom status bar with multiple segments
    - Left segment: File info, line/column, language mode
    - Right segment: Encoding, line endings, compilation status
    - Clickable segments for quick actions
    - Background tasks indicator with progress

**Key Files:**

- `crates/ui/components/button.rs`
- `crates/ui/components/input.rs`
- `crates/ui/components/dropdown.rs`
- `crates/ui/components/tabs.rs`
- `crates/ui/components/splitter.rs`
- `crates/ui/components/scrollbar.rs`
- `crates/ui/components/context_menu.rs`
- `crates/ui/components/tooltip.rs`
- `crates/ui/components/icon.rs`
- `crates/ui/components/status_bar.rs`
- `crates/ui/components/mod.rs` - Component exports and common traits

### 2.4 Main Editor UI Component

**Editor Component Architecture:**

The editor component is the central UI element where users interact with text. It must handle rendering, input, and provide visual feedback for various editor states.

**Component Structure:**

The editor consists of several sub-components working together:

1. **Gutter:**

   - Line numbers (with padding for large files)
   - Git status indicators (added, modified, deleted lines)
   - Breakpoint indicators (for future debugging support)
   - Code folding chevrons
   - Error/warning icons from diagnostics
   - Current line indicator

2. **Text Area:**

   - Text rendering with proper Unicode support
   - Cursor rendering with blink animation (block, line, underline modes)
   - Selection highlighting (single and multi-cursor)
   - Current line highlighting
   - Matching bracket highlighting (configurable)
   - Search result highlighting
   - Whitespace visualization (optional: spaces, tabs, line endings)
   - Indent guides
   - Word wrap support

3. **Scrollable Viewport:**

   - Efficient rendering (only visible lines)
   - Smooth scrolling with pixel-perfect positioning
   - Horizontal scrolling for long lines
   - Virtual scrolling for very large files (1M+ lines)
   - Scroll-to-cursor on navigation
   - Animated scroll for smooth user experience

4. **Minimap (Optional, Collapsible):**

   - Miniature view of entire document
   - Syntax highlighting (simplified color blocks)
   - Viewport indicator showing visible region
   - Click-to-scroll functionality
   - Scroll-to-viewport on drag
   - Toggle visibility

5. **Breadcrumb Navigation:**
   - File path display at top
   - Document structure navigation (sections, functions)
   - Clickable segments for quick navigation
   - Current symbol highlighting

**Integration Points:**

The editor UI must provide clean integration points for backend systems that will be implemented in later phases:

- **Text Buffer Interface:** Abstract interface for text content, allowing mock buffer during UI development
- **LSP Diagnostics Interface:** Callback system for receiving and displaying diagnostics
- **Syntax Highlighting Interface:** Token provider for syntax coloring
- **Bidirectional Text Interface:** Layout calculation for RTL/LTR text rendering
- **Folding Interface:** Fold range provider for collapsible regions
- **Completion Interface:** Trigger and display completion popups

**Visual Features:**

- Current line highlighting with subtle background color
- Matching bracket highlighting when cursor is adjacent to bracket
- Indent guides (vertical lines showing indentation levels)
- Whitespace visualization (optional, configurable):
  - Dots for spaces
  - Arrows for tabs
  - Symbols for line endings
- Code folding UI with smooth expand/collapse animation
- Smooth cursor movement animation (optional, configurable)

**Performance Considerations:**

- Only render visible lines (virtual scrolling)
- Cache rendered glyphs and line measurements
- Batch rendering operations for multiple cursors
- Debounce expensive operations (syntax highlighting, diagnostics)
- Use GPU acceleration for text rendering via GPUI

**Key Files:**

- `crates/ui/editor/editor_view.rs` - Main editor component
- `crates/ui/editor/gutter.rs` - Line numbers and fold indicators
- `crates/ui/editor/text_area.rs` - Text rendering area
- `crates/ui/editor/cursor.rs` - Cursor rendering and animation
- `crates/ui/editor/selection.rs` - Selection rendering
- `crates/ui/editor/minimap.rs` - Minimap component
- `crates/ui/editor/breadcrumb.rs` - File path breadcrumb
- `crates/ui/editor/scroll.rs` - Scroll management
- `crates/ui/editor/viewport.rs` - Visible region calculation

### 2.5 Preview Pane UI

**Preview Component:**

The preview pane displays the rendered Typst document and provides tools for navigation and inspection. It must handle PDF rendering efficiently and provide smooth user experience.

**Component Structure:**

1. **Rendering Viewport:**

   - Canvas for PDF page rendering
   - Support for continuous scroll (pages stacked) or page-by-page mode
   - High-DPI rendering (respect system scale factor)
   - Efficient page caching and lazy loading
   - Background color matching preview background preference

2. **Zoom Controls:**

   - Zoom level display (e.g., "125%")
   - Zoom in/out buttons
   - Zoom slider for fine control
   - Preset zoom levels dropdown:
     - Fit to width
     - Fit to page
     - 50%, 75%, 100%, 125%, 150%, 200%, 300%, 400%
   - Keyboard shortcuts (Ctrl/Cmd + Plus/Minus, Ctrl/Cmd + 0 for reset)
   - Mouse wheel zoom (with Ctrl/Cmd held)

3. **Navigation Controls:**

   - Page indicator (e.g., "Page 3 of 10")
   - Previous/next page buttons
   - Go to page input
   - Thumbnail view for quick navigation (optional)

4. **Toolbar:**

   - Export to PDF button (save compiled PDF)
   - Export to PNG button (render page as image)
   - Print button (system print dialog)
   - Refresh button (force recompilation)
   - Sync indicators (show/hide source position highlights)
   - Settings dropdown (continuous scroll, page mode, etc.)

5. **Sync Visualization:**

   - Highlight regions in preview corresponding to cursor position in source
   - Click on preview to jump to source (backward sync)
   - Animated scroll to synced region
   - Highlight fade-out animation

6. **Status Indicators:**
   - Loading indicator during compilation
   - Compilation progress (if available from compiler)
   - Error state display (compilation failed)
   - Last compilation time
   - Document information (page count, size)

**Integration Points:**

- **PDF Renderer Interface:** Abstract interface for rendering backend
- **Compilation Status Interface:** Receive compilation events (started, completed, failed)
- **Source Sync Interface:** Bidirectional synchronization with editor
- **Export Interface:** Trigger export operations

**User Interactions:**

- Click and drag to pan (when zoomed in)
- Double-click to fit to width
- Ctrl/Cmd + Click to sync to source
- Scroll wheel for vertical navigation
- Shift + Scroll wheel for horizontal navigation
- Ctrl/Cmd + Scroll wheel for zoom

**Key Files:**

- `crates/ui/preview/preview_pane.rs` - Main preview component
- `crates/ui/preview/viewport.rs` - Rendering viewport
- `crates/ui/preview/zoom_controls.rs` - Zoom UI
- `crates/ui/preview/navigation.rs` - Page navigation UI
- `crates/ui/preview/toolbar.rs` - Preview toolbar
- `crates/ui/preview/loading_indicator.rs` - Compilation status display
- `crates/ui/preview/sync_highlight.rs` - Source sync visualization

### 2.6 Sidebar UI

**Sidebar Component:**

The sidebar provides file management and document navigation. It contains multiple views that can be switched using tabs or icons.

**Component Structure:**

1. **File Explorer View:**

   **Features:**

   - Tree view of project files and folders
   - Folder expand/collapse with animation
   - File icons based on file type
   - Indent guides for nested folders
   - Current file highlighting
   - Dirty (unsaved) file indicators

   **Interactions:**

   - Single click to select
   - Double click to open file
   - Drag and drop for file operations (move, copy)
   - Context menu for file operations:
     - New File
     - New Folder
     - Rename (with inline editing)
     - Delete (with confirmation)
     - Copy/Cut/Paste
     - Reveal in system file explorer
     - Copy path

   **Git Integration:**

   - Show Git status indicators (if in Git repository):
     - Modified (M)
     - Added (A)
     - Deleted (D)
     - Untracked (U)
     - Conflict (C)
   - Color coding for Git status

2. **Outline View:**

   **Features:**

   - Tree view of document structure
   - Show document symbols from LSP:
     - Headings (levels 1-6)
     - Sections
     - Functions
     - Variables
     - Labels
   - Icon for each symbol type
   - Indentation showing nesting
   - Current position indicator

   **Interactions:**

   - Click to navigate to symbol
   - Search/filter outline items
   - Expand/collapse nested symbols
   - Auto-expand to show current position
   - Sync selection with cursor position in editor

3. **Search Results View (when active):**

   **Features:**

   - List of search results grouped by file
   - Context preview (line with match highlighted)
   - Match count per file
   - Expandable file groups

   **Interactions:**

   - Click to jump to match
   - Next/previous result navigation

**Sidebar Controls:**

- Tab/icon switcher at top (Explorer, Outline, Search)
- Collapsible sidebar (hide/show with smooth animation)
- Resizable width (drag divider between sidebar and editor)
- Minimum and maximum width constraints

**Key Files:**

- `crates/ui/sidebar/sidebar.rs` - Main sidebar container and tab switching
- `crates/ui/sidebar/file_explorer.rs` - File tree view
- `crates/ui/sidebar/outline.rs` - Document outline view
- `crates/ui/sidebar/search_results.rs` - Search results view
- `crates/ui/sidebar/tree_view.rs` - Reusable tree component
- `crates/ui/sidebar/tree_item.rs` - Tree item rendering

### 2.7 Navigation Bar

**Top Navbar Component:**

The navigation bar provides access to application commands, quick actions. Left - Quick actions, Center - Menu bar, window controls - right. Command pallete opens with an overlay in the middle of the window, Raycast style.

**Component Structure:**

1. **Menu Bar:**

   **Menus:**

   - **File Menu:**

     - New File (Ctrl/Cmd+N)
     - New Window (Ctrl/Cmd+Shift+N)
     - Open File (Ctrl/Cmd+O)
     - Open Folder (Ctrl/Cmd+K, Ctrl/Cmd+O)
     - Open Recent (submenu with recent files)
     - Save (Ctrl/Cmd+S)
     - Save As (Ctrl/Cmd+Shift+S)
     - Save All (Ctrl/Cmd+K, S)
     - Close Editor (Ctrl/Cmd+W)
     - Close Window (Ctrl/Cmd+Shift+W)
     - Exit/Quit (Ctrl/Cmd+Q)

   - **Edit Menu:**

     - Undo (Ctrl/Cmd+Z)
     - Redo (Ctrl/Cmd+Shift+Z or Ctrl/Cmd+Y)
     - Cut (Ctrl/Cmd+X)
     - Copy (Ctrl/Cmd+C)
     - Paste (Ctrl/Cmd+V)
     - Find (Ctrl/Cmd+F)
     - Replace (Ctrl/Cmd+H)
     - Find in Files (Ctrl/Cmd+Shift+F)
     - Toggle Line Comment (Ctrl/Cmd+/)
     - Toggle Block Comment (Ctrl/Cmd+Shift+/)

   - **Selection Menu:**

     - Select All (Ctrl/Cmd+A)
     - Expand Selection (Alt+Shift+Right)
     - Shrink Selection (Alt+Shift+Left)
     - Add Cursor Above (Ctrl/Cmd+Alt+Up)
     - Add Cursor Below (Ctrl/Cmd+Alt+Down)
     - Add Next Occurrence (Ctrl/Cmd+D)
     - Select All Occurrences (Ctrl/Cmd+Shift+L)
     - Column Selection Mode (toggle)

   - **View Menu:**

     - Command Palette (Ctrl/Cmd+Shift+P)
     - Toggle Sidebar (Ctrl/Cmd+B)
     - Toggle Preview (Ctrl/Cmd+K, V)
     - Toggle Console (Ctrl/Cmd+J)
     - Toggle Minimap
     - Zoom In (Ctrl/Cmd+Plus)
     - Zoom Out (Ctrl/Cmd+Minus)
     - Reset Zoom (Ctrl/Cmd+0)
     - Toggle Full Screen (F11)

   - **Go Menu:**

     - Go to File (Ctrl/Cmd+P)
     - Go to Line (Ctrl/Cmd+G)
     - Go to Symbol (Ctrl/Cmd+Shift+O)
     - Go to Definition (F12)
     - Go Back (Alt+Left)
     - Go Forward (Alt+Right)

   - **Tools Menu:**

     - Compile Document (F5)
     - Export to PDF
     - Export to PNG
     - Format Document (Shift+Alt+F)
     - Settings (Ctrl/Cmd+Comma)

   - **Help Menu:**
     - Documentation
     - Typst Documentation
     - Keyboard Shortcuts
     - Report Issue
     - About

2. **Quick Actions Toolbar:**

   **Buttons:**

   - New File (icon button)
   - Open File (icon button)
   - Save (icon button, disabled when no changes)
   - Undo (icon button, disabled when nothing to undo)
   - Redo (icon button, disabled when nothing to redo)
   - Compile (icon button with status indicator)

3. **Command Palette:**

   **Features:**

   - Overlay triggered with Ctrl/Cmd+Shift+P
   - Fuzzy search through all commands
   - Show keyboard shortcuts
   - Recently used commands at top
   - Command categories (File, Edit, View, etc.)

**Key Files:**

- `crates/ui/navbar/navbar.rs` - Main navigation bar
- `crates/ui/navbar/menu.rs` - Menu bar and dropdown menus
- `crates/ui/navbar/toolbar.rs` - Quick actions toolbar
- `crates/ui/navbar/search_box.rs` - Global search input
- `crates/ui/components/command_palette.rs` - Command palette overlay

### 2.8 Console/Diagnostics Panel

**Console Panel Component:**

The bottom panel provides information about compilation, diagnostics, and optionally a terminal. It can be hidden or resized by the user.

**Component Structure:**

1. **Tab Bar:**

   - Problems tab (error/warning count badge)
   - Output tab
   - Terminal tab (optional, future feature)
   - Close button to hide panel

2. **Problems Tab:**

   **Features:**

   - List of all diagnostics (errors, warnings, info, hints)
   - Grouped by file
   - Severity icon and color coding
   - Message preview with full text on hover
   - Source location (file:line:column)
   - Filter by severity (buttons to toggle visibility)
   - Search/filter diagnostics

   **Interactions:**

   - Click diagnostic to jump to source location
   - Context menu for copying diagnostic info
   - Clear diagnostics on successful compilation

3. **Output Tab:**

   **Features:**

   - Compilation output and messages
   - Timestamped entries
   - Syntax highlighting for output
   - Auto-scroll to bottom (with option to pause)
   - Clear button

   **Output Types:**

   - Compilation started
   - Compilation completed (with time)
   - Compilation failed (with error summary)
   - File saved messages
   - LSP initialization messages
   - Debug/info logs (configurable verbosity)

4. **Panel Controls:**

   **Features:**

   - Resize handle at top (drag to resize height)
   - Maximize button (expand to fill editor area)
   - Restore button (return to previous size)
   - Close button (hide panel)
   - Keyboard shortcut to toggle (Ctrl/Cmd+J)

**Key Files:**

- `crates/ui/panels/console/console_panel.rs` - Main console container
- `crates/ui/panels/console/diagnostics_list.rs` - Problems tab
- `crates/ui/panels/console/diagnostic_item.rs` - Individual diagnostic rendering
- `crates/ui/panels/console/output_view.rs` - Output tab
- `crates/ui/panels/console/output_entry.rs` - Output message rendering

### 2.9 Layout & Panel Management

**Panel System:**

A flexible layout system that allows users to customize their workspace by arranging panels in various configurations.

**Layout Manager Features:**

1. **Split View Support:**

   - Horizontal splits (top/bottom)
   - Vertical splits (left/right)
   - Nested splits (split a split for complex layouts)
   - Resize splits by dragging dividers
   - Remove splits by closing all editors in a pane
   - Keyboard shortcuts for split management

2. **Panel Visibility:**

   - Toggle sidebar (Ctrl/Cmd+B)
   - Toggle preview (Ctrl/Cmd+K, V)
   - Toggle console (Ctrl/Cmd+J)
   - Remember visibility state per workspace
   - Zen mode (hide all panels, show only editor)

3. **Layout Presets:**

   Provide quick access to common layouts:

   - **Editor Only:** Just the editor, all panels hidden
   - **Editor + Preview:** Side-by-side editor and preview
   - **Full IDE:** Editor, sidebar, preview, and console
   - **Writing Mode:** Editor with preview below
   - **Custom:** User-defined layouts (save and restore)

4. **State Persistence:**

   - Save layout configuration per workspace
   - Remember panel sizes and positions
   - Restore layout on workspace reopen
   - Export/import layout configurations

5. **Drag and Drop:**
   - Drag tabs between splits
   - Drag tabs to create new splits
   - Drag panels to rearrange
   - Visual feedback during drag operation

**Panel Types:**

Each panel type has a specific purpose and can be placed in various positions:

- **Editor Panel:** The main text editor
- **Preview Panel:** Document preview
- **Sidebar Panel:** File explorer, outline, search
- **Console Panel:** Diagnostics, output, terminal

**Key Files:**

- `crates/ui/panels/panel_manager.rs` - Panel layout management
- `crates/ui/panels/layout.rs` - Layout configuration and persistence
- `crates/ui/panels/split_view.rs` - Split pane implementation
- `crates/ui/panels/panel.rs` - Base panel trait and implementation
- `crates/ui/panels/drag_drop.rs` - Drag-and-drop functionality

---

## Phase 3: Text Buffer & Core Editing

### 3.1 Text Buffer Implementation

**Rope Data Structure:**

The text buffer is the core data structure that holds document content and provides efficient operations for editing. A rope-based implementation is chosen for its optimal performance characteristics with large documents.

**Buffer Requirements:**

- Use rope data structure for efficient text manipulation (consider integrating `ropey` crate)
- Support efficient insertion and deletion at arbitrary positions
- Maintain O(log n) complexity for most operations
- Support efficient slicing for retrieving text ranges
- Maintain line/column indexing with fast lookups
- Handle UTF-8 encoding with proper grapheme cluster awareness
- Support very large files (100MB+) without performance degradation

**Core Operations:**

```rust
pub trait TextBuffer {
    // Content operations
    fn insert(&mut self, position: usize, text: &str);
    fn delete(&mut self, range: Range<usize>);
    fn replace(&mut self, range: Range<usize>, text: &str);
    fn text(&self) -> String;
    fn text_range(&self, range: Range<usize>) -> String;

    // Line operations
    fn line_count(&self) -> usize;
    fn line(&self, line_index: usize) -> Option<String>;
    fn line_range(&self, line_index: usize) -> Option<Range<usize>>;
    fn offset_to_line_col(&self, offset: usize) -> (usize, usize);
    fn line_col_to_offset(&self, line: usize, col: usize) -> usize;

    // Character operations
    fn char_at(&self, offset: usize) -> Option<char>;
    fn grapheme_at(&self, offset: usize) -> Option<&str>;
    fn next_grapheme(&self, offset: usize) -> usize;
    fn prev_grapheme(&self, offset: usize) -> usize;

    // Metadata
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_dirty(&self) -> bool;
    fn set_dirty(&mut self, dirty: bool);
}
```

**Grapheme Cluster Support:**

Proper handling of Unicode grapheme clusters is essential for cursor movement and text selection. A grapheme cluster is a user-perceived character, which may consist of multiple Unicode code points (e.g., emoji with skin tone modifiers, combining diacritics).

- Use `unicode-segmentation` crate for grapheme boundary detection
- Implement cursor movement by grapheme clusters, not code points
- Handle complex scripts (Arabic, Devanagari, etc.) correctly
- Support zero-width joiners and combining characters

**Dirty Tracking:**

- Maintain dirty flag for unsaved changes
- Track dirty state per buffer
- Notify UI when dirty state changes
- Show dirty indicators in tabs and file explorer

**Key Files:**

- `crates/editor_core/buffer/rope_buffer.rs` - Rope-based buffer implementation
- `crates/editor_core/buffer/text_buffer.rs` - Buffer trait and interface
- `crates/editor_core/buffer/operations.rs` - Buffer operation implementations
- `crates/editor_core/buffer/grapheme.rs` - Grapheme cluster utilities
- `crates/editor_core/buffer/line_ending.rs` - Line ending detection and handling

### 3.2 Multi-Cursor Support

**Multi-Cursor Architecture:**

Modern code editors support multiple cursors for simultaneous editing at different locations. This feature significantly improves productivity for repetitive edits.

**Cursor Representation:**

```rust
pub struct Cursor {
    pub anchor: usize,  // Selection start point
    pub head: usize,    // Current cursor position
}

impl Cursor {
    // Selection direction
    pub fn is_forward(&self) -> bool {
        self.head >= self.anchor
    }

    // Selection range
    pub fn range(&self) -> Range<usize> {
        if self.is_forward() {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }

    // Has selection?
    pub fn has_selection(&self) -> bool {
        self.anchor != self.head
    }
}

pub struct MultiCursor {
    cursors: Vec<Cursor>,
    primary: usize,  // Index of primary cursor
}
```

**Multi-Cursor Features:**

1. **Adding Cursors:**

   - Add cursor at click position (Alt/Ctrl + Click)
   - Add cursor above current line (Ctrl/Cmd+Alt+Up)
   - Add cursor below current line (Ctrl/Cmd+Alt+Down)
   - Add cursor at next occurrence of selection (Ctrl/Cmd+D)
   - Add cursors at all occurrences (Ctrl/Cmd+Shift+L)
   - Add cursors at line ends in selection (Shift+Alt+I)

2. **Removing Cursors:**

   - Escape key to remove all but primary cursor
   - Undo last cursor addition (Ctrl/Cmd+U)
   - Click without modifier to reset to single cursor

3. **Cursor Management:**

   - Automatically merge overlapping cursors
   - Maintain cursor order (top to bottom, left to right)
   - Preserve cursor positions during edits
   - Support independent selections for each cursor
   - Primary cursor for operations that affect only one cursor

4. **Column (Rectangular) Selection:**
   - Alt+Shift+Drag for column selection
   - Creates cursors at each line in the selected region
   - Useful for editing tables and aligned code

**Selection Operations:**

- Expand selection to word boundaries (Ctrl/Cmd+D)
- Expand selection to line boundaries (Ctrl/Cmd+L)
- Expand selection to matching brackets (Ctrl/Cmd+Shift+M)
- Shrink selection to previous state (Alt+Shift+Left)
- Select all (Ctrl/Cmd+A)

**Multi-Cursor Editing:**

All text operations must work correctly with multiple cursors:

- Typing inserts text at each cursor
- Delete/backspace operates at each cursor
- Paste pastes the same text at each cursor (or distributes clipboard lines)
- Operations are applied in reverse order to maintain position validity

**Key Files:**

- `crates/editor_core/selection/cursor.rs` - Cursor representation
- `crates/editor_core/selection/multi_cursor.rs` - Multi-cursor management
- `crates/editor_core/selection/selection_mode.rs` - Selection modes (character, line, column)
- `crates/editor_core/selection/operations.rs` - Selection operations

### 3.3 Edit Operations

**Core Edit Operations:**

Text editing operations form the foundation of the editor. These operations must work correctly with multi-cursor selections and maintain buffer consistency.

**Basic Operations:**

1. **Insert Character:**

   - Insert character at each cursor position
   - Move cursor forward after insertion
   - Merge cursors that become adjacent
   - Handle auto-pairing (brackets, quotes)

2. **Delete Operations:**

   - Delete forward (Delete key): Remove character after cursor
   - Delete backward (Backspace): Remove character before cursor
   - Delete word forward (Ctrl/Cmd+Delete)
   - Delete word backward (Ctrl/Cmd+Backspace)
   - Delete line (Ctrl/Cmd+Shift+K)
   - Delete to end of line (Ctrl/Cmd+K, Ctrl/Cmd+K)
   - Delete selection if present

3. **Newline Operations:**

   - Insert newline at cursor (Enter)
   - Auto-indentation: Match indentation of previous line
   - Smart indentation: Increase indent after opening brackets
   - Decrease indent for closing brackets
   - Insert newline above (Ctrl/Cmd+Shift+Enter)
   - Insert newline below (Ctrl/Cmd+Enter)

4. **Tab Operations:**

   - Insert tab or spaces (based on settings)
   - Indent selection (Tab with selection)
   - Outdent selection (Shift+Tab)
   - Smart tab: Align to next tab stop

5. **Auto-Pairing:**
   - Automatically insert closing bracket when typing opening bracket
   - Automatically insert closing quote when typing opening quote
   - Skip closing character if already present
   - Wrap selection in brackets/quotes when typing opener
   - Configurable pairs: (), [], {}, "", '', ``, $$

**Advanced Operations:**

1. **Move Lines:**

   - Move line(s) up (Alt+Up): Swap with line above
   - Move line(s) down (Alt+Down): Swap with line below
   - Preserve indentation and cursor position
   - Work with multi-line selections

2. **Duplicate Lines:**

   - Duplicate line/selection up (Shift+Alt+Up)
   - Duplicate line/selection down (Shift+Alt+Down)
   - Preserve cursor and selection

3. **Join Lines:**

   - Join line with next (Ctrl/Cmd+J)
   - Remove line break and adjust whitespace
   - Smart joining: Single space between joined parts

4. **Comment Toggle:**

   - Toggle line comment (Ctrl/Cmd+/)
   - Toggle block comment (Ctrl/Cmd+Shift+/)
   - Language-aware comment syntax (Typst uses // for line, /\* \*/ for block)
   - Work with multiple selections

5. **Indent/Outdent:**

   - Indent selection (Tab)
   - Outdent selection (Shift+Tab)
   - Respect indentation settings (tabs vs. spaces)
   - Preserve relative indentation

6. **Transform Text:**
   - Convert to uppercase (Ctrl+K, Ctrl+U)
   - Convert to lowercase (Ctrl+K, Ctrl+L)
   - Capitalize words
   - Sort lines (ascending/descending)

**Smart Features:**

- **Auto-indentation:** Maintain indentation level on newline
- **Smart backspace:** Delete indentation as a single unit
- **Auto-closing:** Automatically insert closing brackets/quotes
- **Bracket balancing:** Ensure brackets remain balanced
- **Line ending preservation:** Maintain document's line ending style (LF/CRLF)

**Key Files:**

- `crates/editor_core/operations/insert.rs` - Insertion operations
- `crates/editor_core/operations/delete.rs` - Deletion operations
- `crates/editor_core/operations/newline.rs` - Newline and auto-indent
- `crates/editor_core/operations/transform.rs` - Text transformation
- `crates/editor_core/operations/move_lines.rs` - Move and duplicate lines
- `crates/editor_core/operations/indent.rs` - Indentation operations
- `crates/editor_core/operations/comment.rs` - Comment toggling
- `crates/editor_core/operations/auto_pair.rs` - Auto-pairing logic

### 3.4 Undo/Redo System

**History Management:**

A robust undo/redo system is essential for a good editing experience. The system must handle complex edit scenarios including multi-cursor edits.

**Design:**

Use the Command pattern to encapsulate edit operations as reversible commands.

```rust
pub trait EditCommand {
    fn execute(&self, buffer: &mut dyn TextBuffer, cursors: &mut MultiCursor);
    fn undo(&self, buffer: &mut dyn TextBuffer, cursors: &mut MultiCursor);
    fn merge(&self, other: &dyn EditCommand) -> Option<Box<dyn EditCommand>>;
}

pub struct History {
    undo_stack: Vec<Box<dyn EditCommand>>,
    redo_stack: Vec<Box<dyn EditCommand>>,
    max_size: usize,
    last_save_index: Option<usize>,
}
```

**Features:**

1. **Basic Undo/Redo:**

   - Undo last operation (Ctrl/Cmd+Z)
   - Redo undone operation (Ctrl/Cmd+Shift+Z or Ctrl/Cmd+Y)
   - Maintain undo stack with configurable size limit
   - Clear redo stack when new edit is made

2. **Operation Grouping:**

   - Group consecutive character insertions into single undo step
   - Time-based grouping (e.g., group edits within 1 second)
   - Explicit transaction boundaries for complex operations
   - Break group on cursor movement or selection change

3. **Multi-Cursor Support:**

   - Store all cursor edits as single undoable operation
   - Restore cursor positions on undo/redo
   - Handle cursor merging/splitting correctly

4. **Memory Management:**

   - Configurable maximum undo history size (default: 1000 operations)
   - Discard oldest operations when limit reached
   - Clear history when file is closed (optional)

5. **Save Point Tracking:**
   - Track position in history when file was saved
   - Determine if buffer has unsaved changes
   - Update dirty flag based on save point

**Edit Grouping Strategy:**

- **Character Typing:** Group consecutive character insertions
- **Word Deletion:** Group word deletion as single operation
- **Paste:** Single operation even if multi-line
- **Batch Operations:** Transform, move lines, etc. as single operation
- **Break on:** Cursor movement, selection change, 1-second idle time

**Key Files:**

- `crates/editor_core/buffer/history.rs` - Undo/redo stack management
- `crates/editor_core/buffer/command.rs` - Edit command trait and implementations
- `crates/editor_core/buffer/command_types.rs` - Specific command implementations

### 3.5 Keyboard Navigation

**Cursor Movement:**

Efficient keyboard navigation is essential for productivity. The editor must support standard navigation commands and cursor movement semantics.

**Basic Movement:**

1. **Character Movement:**

   - Left arrow: Move left by one grapheme cluster
   - Right arrow: Move right by one grapheme cluster
   - Respect grapheme boundaries (not code points)
   - Handle bidirectional text correctly (later phase)

2. **Line Movement:**

   - Up arrow: Move up one line
   - Down arrow: Move down one line
   - Preserve column position when possible (virtual column)
   - Snap to line end if line is shorter than target column

3. **Word Movement:**

   - Ctrl/Cmd+Left: Move to start of previous word
   - Ctrl/Cmd+Right: Move to start of next word
   - Alt+Left: Move to start of previous word (macOS)
   - Alt+Right: Move to end of next word (macOS)
   - Word boundaries: Whitespace, punctuation, case changes

4. **Line Boundaries:**

   - Home: Move to first non-whitespace character (or start if already there)
   - End: Move to end of line
   - Ctrl/Cmd+Left: Move to start of line (Windows/Linux)
   - Ctrl/Cmd+Right: Move to end of line (Windows/Linux)

5. **Document Boundaries:**

   - Ctrl/Cmd+Home: Move to start of document
   - Ctrl/Cmd+End: Move to end of document
   - Cmd+Up: Move to start of document (macOS)
   - Cmd+Down: Move to end of document (macOS)

6. **Page Movement:**

   - Page Up: Move up one page (viewport height)
   - Page Down: Move down one page
   - Keep cursor on screen during page moves

7. **Go to Line:**
   - Ctrl/Cmd+G: Open "Go to Line" dialog
   - Enter line number to jump to
   - Support line:column syntax (e.g., "42:15")

**Selection Extension:**

All navigation commands support selection extension with Shift:

- Shift+Arrow: Extend selection
- Shift+Ctrl/Cmd+Arrow: Extend selection by word
- Shift+Home/End: Extend selection to line boundaries
- Shift+Page Up/Down: Extend selection by page

**Navigation Modifiers:**

| Key Combination     | Windows/Linux | macOS        |
| ------------------- | ------------- | ------------ |
| Move left/right     | Arrow         | Arrow        |
| Move word           | Ctrl+Arrow    | Alt+Arrow    |
| Move line start/end | Home/End      | Cmd+Arrow    |
| Move doc start/end  | Ctrl+Home/End | Cmd+Up/Down  |
| Move page           | Page Up/Down  | Page Up/Down |

**Scroll vs. Cursor Movement:**

Some commands scroll the viewport without moving the cursor:

- Ctrl+Up: Scroll viewport up (cursor stays)
- Ctrl+Down: Scroll viewport down
- Center cursor: Ctrl/Cmd+L (scroll to center cursor)

**Key Files:**

- `crates/editor_core/operations/navigation.rs` - Navigation commands
- `crates/editor_core/operations/word_boundary.rs` - Word boundary detection
- `crates/ui/editor/input_handler.rs` - Keyboard input handling and mapping
- `crates/ui/editor/keyboard_map.rs` - Platform-specific keyboard mappings

---

## Phase 4: Bidirectional Text Support

### 4.1 Unicode Bidirectional Algorithm (UAX #9)

**Bidi Algorithm Implementation:**

Bidirectional text support is crucial for properly handling documents containing both left-to-right (LTR) and right-to-left (RTL) scripts. The Unicode Bidirectional Algorithm (UAX #9) defines the standard for mixed-direction text layout.

**Implementation Strategy:**

Rather than implementing the full algorithm from scratch, consider integrating the `unicode-bidi` crate, which provides a robust implementation of UAX #9. However, understanding the algorithm's concepts is essential for proper integration.

**Core Concepts:**

1. **Bidi Character Types:**

   - Strong types: L (left-to-right), R (right-to-left), AL (Arabic letter)
   - Weak types: EN (European number), ES (European separator), ET (European terminator)
   - Neutral types: WS (whitespace), ON (other neutral)
   - Explicit formatting: LRE, RLE, PDF, LRO, RLO, etc.

2. **Embedding Levels:**

   - Base direction (paragraph level)
   - Nested embedding levels for mixed-direction text
   - Even levels = LTR, odd levels = RTL

3. **Algorithm Phases:**
   - P1-P3: Determine paragraph embedding level
   - X1-X10: Resolve explicit embedding levels
   - W1-W7: Resolve weak types
   - N0-N2: Resolve neutral and isolate formatting characters
   - I1-I2: Resolve implicit levels
   - L1-L4: Reorder resolved levels for display

**Integration Requirements:**

```rust
pub struct BidiInfo {
    pub levels: Vec<u8>,           // Embedding level for each byte
    pub paragraph_level: u8,        // Base paragraph direction
    pub runs: Vec<BidiRun>,        // Contiguous runs of same level
}

pub struct BidiRun {
    pub start: usize,
    pub end: usize,
    pub level: u8,
    pub direction: Direction,
}

pub enum Direction {
    Ltr,
    Rtl,
}

pub fn analyze_bidi(text: &str, default_direction: Direction) -> BidiInfo {
    // Use unicode-bidi crate or custom implementation
}
```

**Key Files:**

- `crates/editor_core/bidi_text/algorithm/uax9.rs` - UAX #9 implementation or integration
- `crates/editor_core/bidi_text/algorithm/levels.rs` - Embedding level calculation
- `crates/editor_core/bidi_text/algorithm/reorder.rs` - Visual reordering
- `crates/editor_core/bidi_text/algorithm/types.rs` - Bidi character types

### 4.2 Visual Layout Engine

**Layout Calculation:**

The layout engine translates logical text positions (as stored in the buffer) to visual positions (as displayed on screen) and vice versa.

**Core Responsibilities:**

1. **Logical to Visual Mapping:**

   - Convert buffer offsets to screen coordinates
   - Handle cursor positioning in bidi text
   - Calculate selection rendering regions

2. **Visual to Logical Mapping:**

   - Convert mouse clicks to buffer positions (hit testing)
   - Handle edge cases at direction boundaries

3. **Line Layout:**
   - Split text into visual runs
   - Calculate glyph positions respecting bidi order
   - Handle mixed LTR/RTL content on same line

**Implementation Details:**

```rust
pub struct VisualLine {
    pub runs: Vec<VisualRun>,
    pub direction: Direction,
}

pub struct VisualRun {
    pub text: String,
    pub logical_start: usize,
    pub logical_end: usize,
    pub visual_x: f32,
    pub width: f32,
    pub direction: Direction,
}

impl VisualLine {
    pub fn logical_to_visual(&self, logical_offset: usize) -> f32 {
        // Find run containing offset and calculate visual position
    }

    pub fn visual_to_logical(&self, visual_x: f32) -> usize {
        // Find character at visual position
    }

    pub fn selection_ranges(&self, start: usize, end: usize) -> Vec<(f32, f32)> {
        // Calculate visual ranges for selection (may be discontiguous in bidi text)
    }
}
```

**Cursor Positioning:**

Cursor positioning in bidi text is complex because:

- A single logical position may have two visual positions (at direction boundaries)
- Arrow key movement must respect visual order
- Selection boundaries need special handling

**Dual Cursor Positions:**

At direction boundaries, there are two possible cursor positions:

- **Primary position:** Used for insertion
- **Secondary position:** Shown when navigating across boundary

Example: In text "Hello ×©Ö¸××œ×•Ö¹×", the cursor between "o" and "×©" can appear on either side visually.

**Key Files:**

- `crates/editor_core/bidi_text/layout/layout_engine.rs` - Layout calculation
- `crates/editor_core/bidi_text/layout/visual_line.rs` - Visual line representation
- `crates/editor_core/bidi_text/layout/hit_test.rs` - Mouse position to offset mapping
- `crates/editor_core/bidi_text/layout/selection_ranges.rs` - Selection in bidi text

### 4.3 RTL Line Handling

**Line Direction Detection:**

The editor must detect the primary direction of each line and adjust alignment accordingly. This is particularly important for Typst documents that mix languages.

**Detection Rules:**

1. **First Strong Character:**

   - Scan line from start, ignoring whitespace
   - Find first strong directional character (L, R, or AL)
   - If RTL character found first â†’ line is RTL
   - If LTR character found first â†’ line is LTR

2. **Math Mode Exception:**

   - Typst math mode is denoted by `$...$` for inline or `$ ... $` for block
   - Math content should always be treated as LTR
   - If line starts with math â†’ line is LTR
   - Math takes precedence over language detection

3. **Empty Lines:**
   - Empty lines inherit direction from previous non-empty line
   - If no previous line, use document default (typically LTR)

**Line Alignment:**

Based on detected direction:

- **LTR lines:** Align text from left edge, grow rightward
- **RTL lines:** Align text from right edge, grow leftward
- **Mixed content:** Each run aligns according to its direction within the line

**Implementation:**

```rust
pub enum LineDirection {
    Ltr,
    Rtl,
    Math,  // Special case: always LTR
}

pub struct LineDirectionDetector {
    math_regex: Regex,  // Detect Typst math mode
}

impl LineDirectionDetector {
    pub fn detect(&self, line: &str) -> LineDirection {
        // Check for math mode first
        if self.starts_with_math(line) {
            return LineDirection::Math;
        }

        // Find first strong character
        for ch in line.chars() {
            if ch.is_whitespace() {
                continue;
            }

            match unicode_bidi::bidi_class(ch) {
                BidiClass::R | BidiClass::AL => return LineDirection::Rtl,
                BidiClass::L => return LineDirection::Ltr,
                _ => continue,
            }
        }

        LineDirection::Ltr  // Default
    }

    fn starts_with_math(&self, line: &str) -> bool {
        let trimmed = line.trim_start();
        trimmed.starts_with('$')
    }
}
```

**Visual Implications:**

- Line numbers remain on the left (fixed position)
- Text content aligns based on direction
- Cursor home/end behavior adapts to line direction
- Selection rendering accounts for direction

**Key Files:**

- `crates/editor_core/bidi_text/line_direction.rs` - Line direction detection
- `crates/editor_core/bidi_text/math_detection.rs` - Math mode detection for Typst
- `crates/ui/editor/text_renderer.rs` - RTL-aware text rendering
- `crates/ui/editor/line_layout.rs` - Per-line layout with direction

### 4.4 Inline Bidi Rendering

**Mixed-Direction Rendering:**

Within a single line, text may switch direction multiple times. For example, an English sentence containing an Arabic phrase needs careful rendering.

**Rendering Pipeline:**

1. **Analyze Text:**

   - Run UAX #9 algorithm on line text
   - Get embedding levels and runs

2. **Reorder Runs:**

   - Reorder runs for visual display
   - Maintain mapping between logical and visual order

3. **Shape Text:**

   - Apply font shaping to each run
   - Handle contextual forms (Arabic, Devanagari)
   - Apply ligatures and kerning

4. **Position Glyphs:**

   - Calculate x-position for each glyph
   - Account for run direction
   - Handle zero-width characters

5. **Render:**
   - Draw glyphs at calculated positions
   - Apply syntax highlighting
   - Render selection backgrounds

**Cursor Movement:**

Cursor movement in bidi text requires special handling:

1. **Visual Movement Mode:**

   - Left arrow: Move visually left
   - Right arrow: Move visually right
   - May jump between runs
   - May change logical position non-monotonically

2. **Logical Movement Mode:**
   - Left arrow: Decrease logical position
   - Right arrow: Increase logical position
   - Cursor may jump visually

Most editors use visual movement mode for arrow keys, which is more intuitive for users.

**Word Selection:**

Double-clicking in bidi text should select the word under cursor, respecting:

- Word boundaries in the appropriate script
- Avoid selecting across strong direction changes
- Handle punctuation and spaces correctly

**Selection Rendering:**

Selection in bidi text may not be a single rectangle:

- Calculate selection ranges in logical order
- Convert to visual positions (may be discontiguous)
- Render multiple rectangles if needed

**Key Files:**

- `crates/ui/editor/bidi_renderer.rs` - Bidi text rendering
- `crates/editor_core/bidi_text/cursor_logic.rs` - Cursor movement in bidi text
- `crates/editor_core/bidi_text/text_shaping.rs` - Text shaping integration
- `crates/ui/editor/selection_painter.rs` - Selection rendering with bidi support

---

## Phase 5: LSP Integration

### 5.1 LSP Client Setup

**LSP Client Architecture:**

The Language Server Protocol (LSP) provides language intelligence features like completion, diagnostics, and navigation. The LSP client communicates with the Typst LSP server via JSON-RPC.

**Communication Protocol:**

LSP uses JSON-RPC 2.0 over stdio (standard input/output). Messages are encoded as:

```
Content-Length: 123\r\n
\r\n
{"jsonrpc": "2.0", "method": "...", ...}
```

**Client Implementation:**

```rust
pub struct LspClient {
    process: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    request_id: AtomicU64,
    pending_requests: Arc<RwLock<HashMap<u64, oneshot::Sender<serde_json::Value>>>>,
    notification_handlers: Arc<RwLock<Vec<Box<dyn Fn(String, serde_json::Value) + Send>>>>,
}

impl LspClient {
    pub async fn new(server_path: &str, args: &[String]) -> Result<Self> {
        // Spawn LSP server process
        // Set up stdio communication
        // Start message reading loop
    }

    pub async fn request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        // Send request with unique ID
        // Wait for response
        // Return result or error
    }

    pub fn notify(&self, method: &str, params: serde_json::Value) -> Result<()> {
        // Send notification (no response expected)
    }

    pub fn on_notification<F>(&self, handler: F)
    where
        F: Fn(String, serde_json::Value) + Send + 'static
    {
        // Register notification handler
    }
}
```

**Threading Model:**

- **Main Thread:** UI and application logic
- **LSP Thread:** Dedicated thread for LSP communication
- **Message Loop:** Continuously reads messages from server
- **Request/Response:** Async requests with timeout handling

**Message Handling:**

1. **Outgoing Requests:**

   - Generate unique request ID
   - Send request message
   - Store pending request with response channel
   - Wait for response with timeout

2. **Incoming Responses:**

   - Match response ID to pending request
   - Send result through channel
   - Remove from pending requests

3. **Incoming Notifications:**
   - No ID in message
   - Dispatch to registered handlers
   - Handle asynchronously

**Key Files:**

- `crates/typst_integration/lsp_client/client.rs` - Main LSP client
- `crates/typst_integration/lsp_client/protocol/transport.rs` - JSON-RPC transport
- `crates/typst_integration/lsp_client/protocol/message.rs` - Message types
- `crates/typst_integration/lsp_client/protocol/codec.rs` - Message encoding/decoding

### 5.2 Initialization & Capabilities

**LSP Initialization Sequence:**

1. **Start Server:** Launch Typst LSP server process
2. **Send Initialize:** Send `initialize` request with client capabilities
3. **Receive ServerCapabilities:** Parse server's capabilities
4. **Send Initialized:** Send `initialized` notification
5. **Register Capabilities:** Register for dynamic capabilities if needed

**Client Capabilities:**

```json
{
	"textDocument": {
		"synchronization": {
			"dynamicRegistration": true,
			"willSave": true,
			"willSaveWaitUntil": true,
			"didSave": true
		},
		"completion": {
			"dynamicRegistration": true,
			"completionItem": {
				"snippetSupport": true,
				"commitCharactersSupport": true,
				"documentationFormat": ["markdown", "plaintext"],
				"deprecatedSupport": true,
				"preselectSupport": true
			}
		},
		"hover": {
			"dynamicRegistration": true,
			"contentFormat": ["markdown", "plaintext"]
		},
		"signatureHelp": {
			"dynamicRegistration": true,
			"signatureInformation": {
				"documentationFormat": ["markdown", "plaintext"],
				"parameterInformation": {
					"labelOffsetSupport": true
				}
			}
		},
		"definition": {
			"dynamicRegistration": true,
			"linkSupport": true
		},
		"references": {
			"dynamicRegistration": true
		},
		"documentSymbol": {
			"dynamicRegistration": true,
			"hierarchicalDocumentSymbolSupport": true,
			"symbolKind": {
				"valueSet": [
					1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
					21, 22, 23, 24, 25, 26
				]
			}
		},
		"formatting": {
			"dynamicRegistration": true
		},
		"codeAction": {
			"dynamicRegistration": true,
			"codeActionLiteralSupport": {
				"codeActionKind": {
					"valueSet": [
						"quickfix",
						"refactor",
						"refactor.extract",
						"refactor.inline",
						"refactor.rewrite",
						"source",
						"source.organizeImports"
					]
				}
			}
		}
	},
	"workspace": {
		"applyEdit": true,
		"workspaceEdit": {
			"documentChanges": true,
			"resourceOperations": ["create", "rename", "delete"]
		},
		"didChangeConfiguration": {
			"dynamicRegistration": true
		},
		"didChangeWatchedFiles": {
			"dynamicRegistration": true
		},
		"symbol": {
			"dynamicRegistration": true
		},
		"executeCommand": {
			"dynamicRegistration": true
		},
		"configuration": true,
		"workspaceFolders": true
	}
}
```

**Server Capabilities:**

Parse and store server capabilities to know which features are available:

- `textDocumentSync`: How document changes are synced (none, full, incremental)
- `completionProvider`: Completion support and trigger characters
- `hoverProvider`: Hover information support
- `signatureHelpProvider`: Signature help and trigger characters
- `definitionProvider`: Go to definition support
- `referencesProvider`: Find references support
- `documentSymbolProvider`: Document outline support
- `formattingProvider`: Document formatting support
- `codeActionProvider`: Code actions support

**Key Files:**

- `crates/typst_integration/lsp_client/initialization.rs` - LSP initialization sequence
- `crates/typst_integration/lsp_client/capabilities.rs` - Capability definitions
- `crates/typst_integration/lsp_client/server_capabilities.rs` - Server capability storage

### 5.3 Text Synchronization

**Document Synchronization:**

Keep the LSP server's view of documents in sync with the editor's state.

**Synchronization Events:**

1. **didOpen:**

   ```json
   {
   	"method": "textDocument/didOpen",
   	"params": {
   		"textDocument": {
   			"uri": "file:///path/to/document.typ",
   			"languageId": "typst",
   			"version": 1,
   			"text": "..."
   		}
   	}
   }
   ```

2. **didChange:**

   - **Full sync:** Send entire document content
   - **Incremental sync:** Send only changed regions

   ```json
   {
   	"method": "textDocument/didChange",
   	"params": {
   		"textDocument": {
   			"uri": "file:///path/to/document.typ",
   			"version": 2
   		},
   		"contentChanges": [
   			{
   				"range": {
   					"start": { "line": 10, "character": 5 },
   					"end": { "line": 10, "character": 10 }
   				},
   				"text": "new text"
   			}
   		]
   	}
   }
   ```

3. **didSave:**

   ```json
   {
   	"method": "textDocument/didSave",
   	"params": {
   		"textDocument": {
   			"uri": "file:///path/to/document.typ"
   		},
   		"text": "..." // Optional, if server wants full content
   	}
   }
   ```

4. **didClose:**
   ```json
   {
   	"method": "textDocument/didClose",
   	"params": {
   		"textDocument": {
   			"uri": "file:///path/to/document.typ"
   		}
   	}
   }
   ```

**Version Management:**

- Maintain document version counter
- Increment on each change
- Include version in sync messages
- Handle version mismatches gracefully

**Change Delta Calculation:**

For incremental sync, calculate minimal change deltas:

```rust
pub struct TextChange {
    pub range: Range,
    pub text: String,
}

pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct Position {
    pub line: u32,
    pub character: u32,  // UTF-16 code units, not bytes!
}

fn calculate_changes(old_text: &str, new_text: &str, old_version: u32) -> Vec<TextChange> {
    // Diff old and new text
    // Generate minimal change set
    // Convert byte offsets to line/character positions
    // Remember: LSP uses UTF-16 code units for character positions!
}
```

**UTF-16 Encoding:**

LSP uses UTF-16 code units for positions, not UTF-8 bytes or Unicode code points. Convert carefully:

```rust
fn utf8_to_utf16_offset(text: &str, utf8_offset: usize) -> u32 {
    text[..utf8_offset].encode_utf16().count() as u32
}
```

**Key Files:**

- `crates/typst_integration/lsp_client/sync.rs` - Document synchronization
- `crates/typst_integration/lsp_client/text_delta.rs` - Change delta calculation
- `crates/typst_integration/lsp_client/position.rs` - Position conversion (UTF-8 â†” UTF-16)

### 5.4 Diagnostics

**Receiving Diagnostics:**

The server sends `textDocument/publishDiagnostics` notifications when it finds errors, warnings, or other issues:

```json
{
  "method": "textDocument/publishDiagnostics",
  "params": {
    "uri": "file:///path/to/document.typ",
    "version": 5,
    "diagnostics": [
      {
        "range": {
          "start": {"line": 10, "character": 5},
          "end": {"line": 10, "character": 15}
        },
        "severity": 1,  // 1=Error, 2=Warning, 3=Info, 4=Hint
        "code": "unknown-variable",
        "source": "typst",
        "message": "Unknown variable: my_var",
        "relatedInformation": [
          {
            "location": {
              "uri": "file:///path/to/other.typ",
              "range": {...}
            },
            "message": "Similar variable defined here"
          }
        ]
      }
    ]
  }
}
```

**Diagnostic Storage:**

```rust
pub struct DiagnosticManager {
    diagnostics: HashMap<Url, Vec<Diagnostic>>,
}

impl DiagnosticManager {
    pub fn update(&mut self, uri: Url, diagnostics: Vec<Diagnostic>) {
        self.diagnostics.insert(uri, diagnostics);
        // Notify UI to update
    }

    pub fn get(&self, uri: &Url) -> Option<&[Diagnostic]> {
        self.diagnostics.get(uri).map(|v| v.as_slice())
    }

    pub fn clear(&mut self, uri: &Url) {
        self.diagnostics.remove(uri);
    }
}
```

**Diagnostic UI Rendering:**

1. **In Editor:**

   - Squiggly underlines (wavy lines) under error regions
   - Color-coded by severity (red=error, yellow=warning, blue=info)
   - Error icons in gutter
   - Hover tooltip showing diagnostic message

2. **In Console Panel:**
   - List view of all diagnostics
   - Grouped by file
   - Sortable by severity
   - Click to jump to location
   - Search/filter capabilities

**Diagnostic Overlay:**

```rust
pub fn render_diagnostic_squiggle(
    &self,
    painter: &mut Painter,
    diagnostic: &Diagnostic,
    line_layout: &LineLayout,
) {
    let start_x = line_layout.offset_to_x(diagnostic.range.start);
    let end_x = line_layout.offset_to_x(diagnostic.range.end);
    let y = line_layout.baseline_y + 2.0;

    let color = match diagnostic.severity {
        DiagnosticSeverity::Error => Color::RED,
        DiagnosticSeverity::Warning => Color::YELLOW,
        DiagnosticSeverity::Info => Color::BLUE,
        DiagnosticSeverity::Hint => Color::GRAY,
    };

    paint_wavy_line(painter, start_x, end_x, y, color);
}
```

**Key Files:**

- `crates/typst_integration/diagnostics/diagnostic.rs` - Diagnostic types
- `crates/typst_integration/diagnostics/manager.rs` - Diagnostic storage and management
- `crates/ui/editor/diagnostic_renderer.rs` - Diagnostic UI rendering
- `crates/ui/editor/squiggle_painter.rs` - Wavy underline rendering

### 5.5 Code Completion

**Completion Request:**

Send completion request when user types or explicitly triggers completion:

```json
{
	"method": "textDocument/completion",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		},
		"position": { "line": 10, "character": 15 },
		"context": {
			"triggerKind": 2, // 1=Invoked, 2=TriggerCharacter, 3=TriggerForIncompleteCompletions
			"triggerCharacter": "."
		}
	}
}
```

**Completion Response:**

```json
{
  "isIncomplete": false,
  "items": [
    {
      "label": "text",
      "kind": 3,  // Function
      "detail": "text(content: content) -> content",
      "documentation": {
        "kind": "markdown",
        "value": "Creates a text element..."
      },
      "insertText": "text($1)",
      "insertTextFormat": 2,  // 1=PlainText, 2=Snippet
      "textEdit": {
        "range": {...},
        "newText": "text"
      },
      "additionalTextEdits": [],
      "commitCharacters": ["(", " "],
      "sortText": "0000",
      "filterText": "text",
      "preselect": false
    }
  ]
}
```

**Completion UI:**

1. **Popup Window:**

   - Appears near cursor position
   - Scrollable list of completion items
   - Fuzzy filtering as user types
   - Highlight matching characters

2. **Item Display:**

   - Icon indicating item kind (function, variable, keyword, etc.)
   - Label (item name)
   - Detail text (type signature)
   - Documentation preview (expandable)

3. **Keyboard Navigation:**

   - Up/Down arrows: Navigate list
   - Enter/Tab: Accept selected item
   - Escape: Dismiss popup
   - Continue typing: Filter items

4. **Snippet Expansion:**
   - Parse snippet syntax: `${1:placeholder}`, `${2}`, `$0` (final position)
   - Insert snippet text
   - Create tab stops for placeholders
   - Highlight current placeholder
   - Tab to move to next placeholder

**Completion Triggers:**

- User types trigger character (e.g., `.`, `#`, `:`)
- User presses Ctrl+Space
- Automatic after short delay (configurable)

**Fuzzy Matching:**

Implement fuzzy matching for filtering:

- Match characters in order but not necessarily consecutive
- Score matches based on proximity and position
- Prefer matches at word boundaries

**Key Files:**

- `crates/typst_integration/lsp_client/requests/completion.rs` - Completion requests
- `crates/ui/editor/completion_popup.rs` - Completion UI
- `crates/ui/editor/completion_item.rs` - Completion item rendering
- `crates/editor_core/snippets/expander.rs` - Snippet expansion (reused)

### 5.6 Hover Information

**Hover Request:**

Send hover request when user hovers over a symbol:

```json
{
	"method": "textDocument/hover",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		},
		"position": { "line": 10, "character": 15 }
	}
}
```

**Hover Response:**

````json
{
	"contents": {
		"kind": "markdown",
		"value": "```typst\nfn text(content: content) -> content\n```\n\nCreates a text element with the specified content."
	},
	"range": {
		"start": { "line": 10, "character": 10 },
		"end": { "line": 10, "character": 20 }
	}
}
````

**Hover UI:**

1. **Tooltip Popup:**

   - Appears near cursor/mouse position
   - Renders markdown content
   - Syntax highlighting for code blocks
   - Auto-sized to content
   - Positioned to stay within window bounds

2. **Timing:**

   - Show after hover delay (e.g., 500ms)
   - Hide when mouse moves away
   - Hide when user starts typing
   - Hide on Escape key

3. **Markdown Rendering:**
   - Support basic markdown: bold, italic, code, links
   - Syntax highlighting for code blocks
   - Respect theme colors

**Key Files:**

- `crates/typst_integration/lsp_client/requests/hover.rs` - Hover requests
- `crates/ui/editor/hover_tooltip.rs` - Hover tooltip rendering
- `crates/ui/components/markdown_renderer.rs` - Markdown rendering

### 5.7 Go to Definition & References

**Go to Definition:**

```json
{
	"method": "textDocument/definition",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		},
		"position": { "line": 10, "character": 15 }
	}
}
```

Response: Single location or array of locations

**Find References:**

```json
{
	"method": "textDocument/references",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		},
		"position": { "line": 10, "character": 15 },
		"context": {
			"includeDeclaration": true
		}
	}
}
```

Response: Array of locations

**Location:**

```json
{
	"uri": "file:///path/to/document.typ",
	"range": {
		"start": { "line": 5, "character": 10 },
		"end": { "line": 5, "character": 20 }
	}
}
```

**Navigation UI:**

1. **Go to Definition:**

   - F12 or Ctrl/Cmd+Click
   - Jump to definition location
   - Add to navigation history
   - If multiple definitions, show picker

2. **Find References:**

   - Show results in sidebar panel
   - Group by file
   - Show context (line with match highlighted)
   - Click to navigate
   - Keyboard shortcuts for next/previous

3. **Peek Definition:**

   - Alt+F12 or hover with modifier
   - Show inline popup with definition
   - Edit in peek window
   - Full navigation within peek

4. **Navigation History:**
   - Track navigation jumps
   - Go back (Alt+Left)
   - Go forward (Alt+Right)
   - Show history in dropdown

**Key Files:**

- `crates/typst_integration/lsp_client/requests/navigation.rs` - Navigation requests
- `crates/ui/editor/peek_view.rs` - Peek definition view
- `crates/ui/sidebar/references_panel.rs` - References results panel
- `crates/editor_core/navigation/history.rs` - Navigation history

### 5.8 Signature Help

**Signature Help Request:**

```json
{
	"method": "textDocument/signatureHelp",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		},
		"position": { "line": 10, "character": 20 },
		"context": {
			"triggerKind": 2, // 1=Invoked, 2=TriggerCharacter, 3=ContentChange
			"triggerCharacter": "(",
			"isRetrigger": false,
			"activeSignatureHelp": null
		}
	}
}
```

**Signature Help Response:**

```json
{
	"signatures": [
		{
			"label": "text(content: content, fill: color, size: length) -> content",
			"documentation": "Creates a text element",
			"parameters": [
				{
					"label": [5, 20], // Offset in signature label
					"documentation": "The text content"
				},
				{
					"label": [22, 34],
					"documentation": "Text color"
				},
				{
					"label": [36, 48],
					"documentation": "Font size"
				}
			]
		}
	],
	"activeSignature": 0,
	"activeParameter": 1
}
```

**Signature UI:**

1. **Popup Display:**

   - Show above or below cursor
   - Display function signature
   - Highlight current parameter
   - Show parameter documentation

2. **Multiple Overloads:**

   - Show arrow buttons for overloads
   - Display "1 of 3" indicator
   - Keyboard shortcuts to cycle

3. **Parameter Highlighting:**

   - Bold or different color for active parameter
   - Underline or box around parameter

4. **Auto-Update:**
   - Update on cursor movement
   - Update on typing
   - Dismiss when leaving function call

**Key Files:**

- `crates/typst_integration/lsp_client/requests/signature_help.rs` - Signature requests
- `crates/ui/editor/signature_popup.rs` - Signature UI

### 5.9 Document Symbols & Outline

**Document Symbol Request:**

```json
{
	"method": "textDocument/documentSymbol",
	"params": {
		"textDocument": {
			"uri": "file:///path/to/document.typ"
		}
	}
}
```

**Response (Hierarchical):**

```json
[
  {
    "name": "Introduction",
    "kind": 15,  // SymbolKind (1=File, 2=Module, 5=Class, 6=Method, 12=Function, 15=String, etc.)
    "range": {...},
    "selectionRange": {...},
    "children": [
      {
        "name": "Background",
        "kind": 15,
        "range": {...},
        "selectionRange": {...},
        "children": []
      }
    ]
  }
]
```

**Symbol Kinds:**

- 1: File
- 2: Module
- 3: Namespace
- 4: Package
- 5: Class
- 6: Method
- 7: Property
- 8: Field
- 9: Constructor
- 10: Enum
- 11: Interface
- 12: Function
- 13: Variable
- 14: Constant
- 15: String (often used for headings in Typst)
- 16: Number
- 17: Boolean
- 18: Array
- 19-26: Others

**Outline UI:**

1. **Tree View:**

   - Hierarchical tree of symbols
   - Indentation showing nesting
   - Icons for symbol kinds
   - Expand/collapse nodes

2. **Current Symbol Highlighting:**

   - Highlight symbol containing cursor
   - Auto-expand to show current symbol
   - Optional auto-scroll to visible

3. **Navigation:**

   - Click to jump to symbol
   - Keyboard navigation (arrows)
   - Search/filter symbols

4. **Breadcrumb Integration:**
   - Show current symbol path in breadcrumb
   - Click breadcrumb segment to navigate

**Key Files:**

- `crates/typst_integration/lsp_client/requests/symbols.rs` - Symbol requests
- `crates/ui/sidebar/outline_builder.rs` - Build outline tree from symbols
- `crates/ui/editor/breadcrumb.rs` - Breadcrumb integration

---

## Phase 6: Typst Compiler Integration

### 6.1 Compiler Wrapper

**Typst Compilation:**

Integrate the Typst compiler library to compile documents to PDF. The compiler must run in the background without blocking the UI.

**Compiler Interface:**

```rust
pub struct TypstCompiler {
    world: Arc<RwLock<EditorWorld>>,
    cache: CompilationCache,
}

impl TypstCompiler {
    pub async fn compile(&self, path: &Path) -> Result<CompiledDocument> {
        // Load document and dependencies
        // Run compilation
        // Generate PDF
        // Return result with diagnostics
    }

    pub fn cancel(&self) {
        // Cancel ongoing compilation
    }
}

pub struct CompiledDocument {
    pub pdf: Vec<u8>,
    pub diagnostics: Vec<Diagnostic>,
    pub warnings: Vec<Warning>,
    pub duration: Duration,
}
```

**World Implementation:**

Typst requires a "world" implementation that provides file access and package management:

```rust
pub struct EditorWorld {
    root: PathBuf,
    main_file: PathBuf,
    files: HashMap<PathBuf, FileContent>,
    packages: PackageRegistry,
}

impl typst::World for EditorWorld {
    fn file(&self, path: &Path) -> Result<typst::FileId> {
        // Resolve file path
        // Load file content (from cache or disk)
        // Return file ID
    }

    fn source(&self, id: typst::FileId) -> Result<String> {
        // Return file content
    }

    fn book(&self) -> &typst::Preamble {
        // Return document preamble (bibliography, etc.)
    }
}
```

**Incremental Compilation:**

- Cache parsed syntax trees
- Cache evaluated layout
- Only recompile changed parts
- Significant speedup for large documents

**Key Files:**

- `crates/typst_integration/compiler/compiler.rs` - Compiler interface
- `crates/typst_integration/world/world.rs` - World implementation
- `crates/typst_integration/world/file_cache.rs` - File content caching
- `crates/typst_integration/world/package_manager.rs` - Package management
- `crates/typst_integration/compiler/cache.rs` - Compilation cache

### 6.2 Compilation Triggers

**Auto-Compilation Strategy:**

Balance responsiveness with resource usage:

1. **On Save:** Always compile when user explicitly saves
2. **On Idle:** Compile after typing stops for delay period (e.g., 500ms)
3. **Manual:** User can trigger compilation with F5 or button

**Debouncing:**

```rust
pub struct CompilationDebouncer {
    delay: Duration,
    timer: Option<Task>,
    pending: Arc<AtomicBool>,
}

impl CompilationDebouncer {
    pub fn trigger(&mut self, callback: impl FnOnce() + Send + 'static) {
        // Cancel existing timer
        if let Some(timer) = self.timer.take() {
            timer.cancel();
        }

        // Start new timer
        self.pending.store(true, Ordering::Release);
        self.timer = Some(spawn_after(self.delay, move || {
            if self.pending.load(Ordering::Acquire) {
                callback();
                self.pending.store(false, Ordering::Release);
            }
        }));
    }

    pub fn cancel(&mut self) {
        self.pending.store(false, Ordering::Release);
        if let Some(timer) = self.timer.take() {
            timer.cancel();
        }
    }
}
```

**Cancellation:**

- When new compilation triggered, cancel ongoing compilation
- Check cancellation flag periodically during compilation
- Clean up resources on cancellation

**UI Feedback:**

- Show compilation status in status bar
- Progress indicator during compilation
- Show compilation time on completion
- Error notification on failure

**Key Files:**

- `crates/typst_integration/compiler/trigger.rs` - Compilation triggers
- `crates/typst_integration/compiler/debouncer.rs` - Debounce logic
- `crates/typst_integration/compiler/scheduler.rs` - Compilation scheduling

### 6.3 Error Handling

**Compiler Diagnostics:**

Typst compiler produces diagnostics separate from LSP diagnostics. These need to be converted to a common format and displayed.

**Diagnostic Types:**

1. **Syntax Errors:** Parsing failures
2. **Type Errors:** Type mismatches
3. **Runtime Errors:** Evaluation errors
4. **Warnings:** Non-fatal issues

**Error Conversion:**

```rust
fn convert_typst_diagnostic(diag: &typst::Diagnostic) -> Diagnostic {
    Diagnostic {
        range: convert_span_to_range(&diag.span),
        severity: match diag.severity {
            typst::Severity::Error => DiagnosticSeverity::Error,
            typst::Severity::Warning => DiagnosticSeverity::Warning,
        },
        source: "typst-compiler".to_string(),
        message: diag.message.clone(),
        related_information: diag.hints.iter().map(convert_hint).collect(),
    }
}
```

**Diagnostic Merging:**

Combine LSP and compiler diagnostics:

- Avoid duplicates (same location and message)
- Prefer compiler diagnostics for compilation errors
- Prefer LSP diagnostics for real-time errors
- Show both if different

**Error Traces:**

For nested errors, show full trace:

- Root cause
- Stack of error locations
- Helpful hints and suggestions

**Key Files:**

- `crates/typst_integration/diagnostics/compiler_errors.rs` - Error conversion
- `crates/typst_integration/diagnostics/merger.rs` - Diagnostic deduplication
- `crates/typst_integration/diagnostics/trace.rs` - Error trace formatting

### 6.4 File Watching

**File System Monitoring:**

Watch imported files and assets for external changes:

```rust
pub struct FileWatcher {
    watcher: RecommendedWatcher,
    watched_files: HashSet<PathBuf>,
    debouncer: Debouncer,
}

impl FileWatcher {
    pub fn watch(&mut self, path: &Path) -> Result<()> {
        if self.watched_files.insert(path.to_path_buf()) {
            self.watcher.watch(path, RecursiveMode::NonRecursive)?;
        }
        Ok(())
    }

    pub fn unwatch(&mut self, path: &Path) -> Result<()> {
        if self.watched_files.remove(path) {
            self.watcher.unwatch(path)?;
        }
        Ok(())
    }

    fn on_change(&mut self, event: Event) {
        // Debounce rapid changes
        // Trigger recompilation
        // Update file cache
    }
}
```

**Watch Strategy:**

- Watch main document
- Watch imported files (detected during compilation)
- Watch asset files (images, data files)
- Watch package files (if local packages)

**Debouncing:**

- Multiple rapid changes should trigger single recompilation
- Use short delay (e.g., 100ms) to batch changes

**Handling Changes:**

1. **File Modified:** Recompile if needed
2. **File Deleted:** Show error, offer to close document
3. **File Renamed:** Update references

**Key Files:**

- `crates/typst_integration/compiler/file_watcher.rs` - File watching
- `crates/typst_integration/world/file_cache.rs` - Cache invalidation on changes

---

## Phase 7: Native PDF Preview Rendering

### 7.1 PDF Rendering Library Integration

**Native PDF Rendering:**

Use a native Rust PDF rendering library for best performance. Options:

- `pdfium-render`: Bindings to PDFium (Chrome's PDF library)
- `pdf-rs`: Pure Rust PDF parser and renderer
- `mupdf-rs`: Bindings to MuPDF

**Recommendation:** `pdfium-render` for robustness and performance.

**Renderer Interface:**

```rust
pub struct PdfRenderer {
    backend: Box<dyn PdfBackend>,
    page_cache: LruCache<(usize, f32), RenderedPage>,
}

pub trait PdfBackend: Send + Sync {
    fn load_document(&self, data: &[u8]) -> Result<Box<dyn PdfDocument>>;
}

pub trait PdfDocument: Send + Sync {
    fn page_count(&self) -> usize;
    fn page(&self, index: usize) -> Result<Box<dyn PdfPage>>;
}

pub trait PdfPage: Send + Sync {
    fn size(&self) -> (f32, f32);
    fn render(&self, scale: f32) -> Result<Image>;
}

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,  // RGBA format
}
```

**Rendering Parameters:**

- Scale factor: For high-DPI displays (1x, 2x, 3x)
- Page index: Which page to render
- Region: Optional sub-region for partial rendering

**Caching Strategy:**

```rust
pub struct PageCache {
    cache: LruCache<CacheKey, RenderedPage>,
    max_memory: usize,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    page_index: usize,
    scale: OrderedFloat<f32>,
    region: Option<(u32, u32, u32, u32)>,
}

pub struct RenderedPage {
    image: Image,
    timestamp: Instant,
}
```

**Cache Eviction:**

- LRU eviction when memory limit reached
- Prioritize visible pages
- Pre-render adjacent pages

**Tile-Based Rendering:**

For very large pages, render in tiles:

- Divide page into 512x512 pixel tiles
- Render only visible tiles
- Cache tiles independently

**Key Files:**

- `crates/preview/renderer/pdf_backend.rs` - PDF backend trait
- `crates/preview/renderer/pdfium_backend.rs` - PDFium implementation
- `crates/preview/renderer/page_cache.rs` - Page rendering cache
- `crates/preview/renderer/tile_renderer.rs` - Tile-based rendering
- `crates/preview/renderer/image.rs` - Image representation

### 7.2 Viewport Management

**Viewport Calculations:**

Determine which pages and regions are visible:

```rust
pub struct Viewport {
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub width: f32,
    pub height: f32,
    pub zoom: f32,
}

impl Viewport {
    pub fn visible_pages(&self, page_sizes: &[(f32, f32)]) -> Vec<VisiblePage> {
        // Calculate which pages intersect viewport
        // Return page indices and visible regions
    }
}

pub struct VisiblePage {
    pub index: usize,
    pub visible_rect: Rect,
}
```

**Zoom Levels:**

Predefined zoom levels:

- Fit to width
- Fit to page (entire page visible)
- 50%, 75%, 100%, 125%, 150%, 200%, 300%, 400%
- Custom percentage

**Zoom Implementation:**

```rust
pub enum ZoomMode {
    FitWidth,
    FitPage,
    Custom(f32),  // Percentage (1.0 = 100%)
}

impl ZoomMode {
    pub fn calculate_scale(&self, page_size: (f32, f32), viewport: &Viewport) -> f32 {
        match self {
            ZoomMode::FitWidth => viewport.width / page_size.0,
            ZoomMode::FitPage => {
                let scale_x = viewport.width / page_size.0;
                let scale_y = viewport.height / page_size.1;
                scale_x.min(scale_y)
            }
            ZoomMode::Custom(pct) => pct,
        }
    }
}
```

**Scroll Management:**

- Smooth scrolling with momentum
- Keyboard navigation (arrows, page up/down)
- Mouse wheel scrolling
- Click and drag panning (when zoomed in)

**Continuous Scroll Mode:**

Stack all pages vertically with gaps:

- Calculate total height
- Position pages sequentially
- Smooth scrolling across page boundaries

**Page-by-Page Mode:**

Show one page at a time:

- Next/previous page buttons
- Page number input
- Keyboard shortcuts (Page Up/Down)

**Key Files:**

- `crates/preview/viewport/viewport.rs` - Viewport calculations
- `crates/preview/viewport/scroll_manager.rs` - Scroll handling
- `crates/preview/viewport/zoom.rs` - Zoom management
- `crates/preview/viewport/layout.rs` - Page layout calculation

### 7.3 Source-Preview Synchronization

**Forward Sync (Source â†’ Preview):**

Jump from source position to corresponding preview position:

```rust
pub struct SyncManager {
    source_map: Option<SourceMap>,
}

pub struct SourceMap {
    mappings: Vec<SyncMapping>,
}

pub struct SyncMapping {
    pub source_range: Range<usize>,
    pub page: usize,
    pub rect: Rect,
}

impl SyncManager {
    pub fn source_to_preview(&self, offset: usize) -> Option<PreviewLocation> {
        // Find mapping containing offset
        // Return page and rectangle
    }
}

pub struct PreviewLocation {
    pub page: usize,
    pub rect: Rect,
}
```

**Backward Sync (Preview â†’ Source):**

Jump from preview position to source:

```rust
impl SyncManager {
    pub fn preview_to_source(&self, page: usize, point: (f32, f32)) -> Option<usize> {
        // Find mappings on page containing point
        // Return source offset
    }
}
```

**Source Mapping Generation:**

Typst compiler can provide source mappings if instrumented:

- Track source spans during compilation
- Map spans to output positions
- Store mapping data in document or separately

**Fallback Heuristics:**

If precise mapping unavailable:

- Estimate position based on line/page ratio
- Use document structure (headings) as anchors
- Approximate based on character count

**Sync Visualization:**

1. **Forward Sync:**

   - Scroll preview to show corresponding region
   - Highlight region with colored overlay
   - Fade out highlight after 1-2 seconds

2. **Backward Sync:**
   - Jump editor to corresponding line
   - Highlight line briefly
   - Center line in viewport

**User Interaction:**

- Ctrl/Cmd+Click in source: Jump to preview
- Ctrl/Cmd+Click in preview: Jump to source
- Auto-sync on cursor move (optional, configurable)

**Key Files:**

- `crates/preview/sync/sync_manager.rs` - Sync coordination
- `crates/preview/sync/source_map.rs` - Source location mapping
- `crates/preview/sync/highlight.rs` - Sync highlight rendering
- `crates/typst_integration/compiler/source_mapping.rs` - Generate source maps

### 7.4 Preview Updates

**Handling Compilation Results:**

When compilation completes, update preview:

```rust
pub struct PreviewUpdateManager {
    current_document: Option<Vec<u8>>,
    current_page_count: usize,
}

impl PreviewUpdateManager {
    pub async fn update(&mut self, pdf: Vec<u8>) -> Result<UpdateResult> {
        // Compare with current document
        // Determine which pages changed (if possible)
        // Update preview
        // Preserve scroll position
    }
}

pub struct UpdateResult {
    pub changed_pages: Option<Vec<usize>>,
    pub should_preserve_scroll: bool,
}
```

**Scroll Preservation:**

Try to maintain user's view when updating:

- Remember current page and position
- After update, scroll back to same page
- If page count changed, adjust intelligently

**Change Detection:**

Detect which pages changed:

- Compare PDF page by page (if possible)
- Only re-render changed pages
- Faster updates for small edits

**Transition Effects:**

Smooth transition on update:

- Fade effect (optional)
- Flash to indicate update
- Cross-fade between old and new

**Loading States:**

Clear indication of compilation state:

- Compiling: Show spinner or progress bar
- Success: Update preview, show completion message
- Error: Show error indicator, keep old preview

**Key Files:**

- `crates/preview/renderer/update_manager.rs` - Preview update handling
- `crates/preview/renderer/transition.rs` - Smooth transitions
- `crates/preview/renderer/change_detection.rs` - Detect changed pages

### 7.5 Export & Print

**Export Features:**

1. **Export to PDF:**

   - Save compiled PDF to custom location
   - Choose filename and directory
   - Option to open after export

2. **Export to PNG/JPEG:**

   - Render pages as raster images
   - Configurable resolution/DPI
   - Export all pages or selection
   - Batch export with filename template

3. **Export to SVG:**
   - If Typst supports SVG output
   - Vector format for scalability

**Export Implementation:**

```rust
pub struct Exporter {
    renderer: Arc<PdfRenderer>,
}

impl Exporter {
    pub async fn export_pdf(&self, pdf: &[u8], path: &Path) -> Result<()> {
        // Write PDF to file
        fs::write(path, pdf).await?;
        Ok(())
    }

    pub async fn export_images(
        &self,
        pdf: &[u8],
        pages: Range<usize>,
        dpi: u32,
        format: ImageFormat,
        output_template: &str,
    ) -> Result<Vec<PathBuf>> {
        // Render pages at high resolution
        // Save as image files
        // Return list of created files
    }
}

pub enum ImageFormat {
    Png,
    Jpeg { quality: u8 },
}
```

**Print Integration:**

Integrate with system print dialog:

```rust
pub fn print_document(&self, pdf: &[u8]) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        // Use Windows printing API
    }

    #[cfg(target_os = "macos")]
    {
        // Use macOS printing API
    }

    #[cfg(target_os = "linux")]
    {
        // Use CUPS or other Linux printing system
    }
}
```

**Export UI:**

- Export dialog with options
- Progress indicator for batch export
- Success notification with "Open" button
- Error handling and user feedback

**Key Files:**

- `crates/preview/export/exporter.rs` - Export functionality
- `crates/preview/export/image_export.rs` - Image export
- `crates/preview/export/print.rs` - Print integration
- `crates/ui/preview/export_dialog.rs` - Export UI dialog

---

## Phase 8: Advanced Editor Features

### 8.1 Search & Replace

**Find in Document:**

```rust
pub struct SearchEngine {
    pub query: String,
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub regex: bool,
    pub matches: Vec<SearchMatch>,
}

pub struct SearchMatch {
    pub range: Range<usize>,
    pub line: usize,
}

impl SearchEngine {
    pub fn search(&mut self, text: &str) -> Result<()> {
        self.matches.clear();

        if self.regex {
            self.search_regex(text)?;
        } else {
            self.search_literal(text);
        }

        Ok(())
    }

    fn search_literal(&mut self, text: &str) {
        // Implement literal search with options
    }

    fn search_regex(&mut self, text: &str) -> Result<()> {
        // Compile and execute regex
        // Validate regex syntax
    }
}
```

**Search UI:**

1. **Search Box:**

   - Text input for search query
   - Match count display (e.g., "3 of 15")
   - Previous/next match buttons
   - Close button
   - Options toggles (case, word, regex)

2. **Search Highlighting:**

   - Highlight all matches in editor
   - Current match with different color
   - Update highlights as user types
   - Clear highlights when search closed

3. **Navigation:**
   - F3 or Enter: Next match
   - Shift+F3 or Shift+Enter: Previous match
   - Wrap around at document boundaries

**Replace Functionality:**

```rust
pub struct ReplaceEngine {
    pub search: SearchEngine,
    pub replacement: String,
}

impl ReplaceEngine {
    pub fn replace_current(&self, buffer: &mut TextBuffer, match_index: usize) {
        let range = self.search.matches[match_index].range.clone();
        buffer.replace(range, &self.replacement);
    }

    pub fn replace_all(&self, buffer: &mut TextBuffer) {
        // Replace in reverse order to maintain positions
        for m in self.search.matches.iter().rev() {
            buffer.replace(m.range.clone(), &self.replacement);
        }
    }
}
```

**Replace UI:**

- Replace input field
- Replace button (current match)
- Replace All button
- Confirmation dialog for Replace All
- Undo replaces as single operation

**Regex Capture Groups:**

Support $1, $2, etc. in replacement:

```
Search: (\w+)\s+(\w+)
Replace: $2 $1
```

**Global Search (Find in Files):**

```rust
pub struct WorkspaceSearch {
    pub query: SearchQuery,
    pub results: Vec<FileSearchResult>,
}

pub struct FileSearchResult {
    pub path: PathBuf,
    pub matches: Vec<SearchMatch>,
}

impl WorkspaceSearch {
    pub async fn search(&mut self, workspace_root: &Path) -> Result<()> {
        // Walk directory tree
        // Search each file
        // Collect results
        // Respect .gitignore
    }
}
```

**Global Search UI:**

- Search input with file filter
- Results grouped by file
- Show match context (line text)
- Click to jump to match
- Replace in files support

**Key Files:**

- `crates/editor_core/search/search_engine.rs` - Search logic
- `crates/editor_core/search/regex.rs` - Regex support
- `crates/editor_core/search/replace.rs` - Replace logic
- `crates/ui/editor/search_box.rs` - Search UI
- `crates/ui/editor/search_highlight.rs` - Match highlighting
- `crates/ui/workspace/global_search.rs` - Workspace search
- `crates/ui/workspace/search_results.rs` - Search results panel

### 8.2 Code Folding

**Fold Detection:**

```rust
pub enum FoldMethod {
    Lsp,          // Use LSP folding ranges
    Indentation,  // Indentation-based folding
    Syntax,       // Syntax-aware (future)
}

pub struct FoldDetector {
    method: FoldMethod,
}

pub struct FoldRange {
    pub start_line: usize,
    pub end_line: usize,
    pub kind: FoldKind,
}

pub enum FoldKind {
    Comment,
    Imports,
    Region,
    Function,
    Block,
}

impl FoldDetector {
    pub fn detect_folds(&self, buffer: &TextBuffer) -> Vec<FoldRange> {
        match self.method {
            FoldMethod::Lsp => self.get_lsp_folds(),
            FoldMethod::Indentation => self.detect_indentation_folds(buffer),
            FoldMethod::Syntax => self.detect_syntax_folds(buffer),
        }
    }
}
```

**Fold State Management:**

```rust
pub struct FoldManager {
    folds: HashMap<usize, Fold>,  // Key: start line
}

pub struct Fold {
    pub range: FoldRange,
    pub is_folded: bool,
}

impl FoldManager {
    pub fn toggle(&mut self, line: usize) {
        if let Some(fold) = self.folds.get_mut(&line) {
            fold.is_folded = !fold.is_folded;
        }
    }

    pub fn fold_all(&mut self) {
        for fold in self.folds.values_mut() {
            fold.is_folded = true;
        }
    }

    pub fn unfold_all(&mut self) {
        for fold in self.folds.values_mut() {
            fold.is_folded = false;
        }
    }
}
```

**Fold Rendering:**

1. **Gutter Indicators:**

   - Chevron icon (â–¼ when expanded, â–¶ when folded)
   - Show on lines with foldable regions
   - Click to toggle fold
   - Hover to highlight fold region

2. **Folded Region:**

   - Replace folded lines with placeholder
   - Show line count (e.g., "... (15 lines)")
   - Click placeholder to unfold
   - Gray background for placeholder

3. **Line Number Adjustment:**
   - Hide line numbers for folded lines
   - Show continuous numbering

**Keyboard Shortcuts:**

- Ctrl/Cmd+Shift+[: Fold current region
- Ctrl/Cmd+Shift+]: Unfold current region
- Ctrl/Cmd+K, Ctrl/Cmd+0: Fold all
- Ctrl/Cmd+K, Ctrl/Cmd+J: Unfold all
- Ctrl/Cmd+K, Ctrl/Cmd+1-9: Fold level 1-9

**Persistence:**

- Save fold state per document
- Restore folds when reopening file

**Key Files:**

- `crates/editor_core/folding/fold_manager.rs` - Fold state management
- `crates/editor_core/folding/fold_detector.rs` - Fold region detection
- `crates/editor_core/folding/indentation.rs` - Indentation-based folding
- `crates/ui/editor/fold_gutter.rs` - Fold UI in gutter
- `crates/ui/editor/fold_placeholder.rs` - Folded region rendering

### 8.3 Code Snippets

**Snippet Format:**

Use LSP snippet syntax:

```
fn ${1:name}(${2:param}: ${3:type}) -> ${4:type} {
    ${5:body}$0
}
```

- `$1`, `$2`, etc.: Tab stops
- `${1:placeholder}`: Tab stop with placeholder text
- `$0`: Final cursor position
- `${1|option1,option2|}`: Choice (dropdown)
- Variables: `$TM_FILENAME`, `$CURRENT_YEAR`, etc.

**Snippet Definition:**

```json
{
	"Document Template": {
		"prefix": "doc",
		"body": [
			"#set document(title: \"${1:Title}\", author: \"${2:Author}\")",
			"#set page(paper: \"${3|a4,us-letter,a5|}\")",
			"",
			"= ${1:Title}",
			"",
			"$0"
		],
		"description": "Basic Typst document template"
	}
}
```

**Snippet Parser:**

```rust
pub struct Snippet {
    pub trigger: String,
    pub body: Vec<String>,
    pub description: String,
    pub tab_stops: Vec<TabStop>,
}

pub struct TabStop {
    pub index: usize,
    pub placeholder: Option<String>,
    pub choices: Option<Vec<String>>,
    pub transforms: Vec<Transform>,
}

pub fn parse_snippet(text: &str) -> Result<Snippet> {
    // Parse snippet syntax
    // Extract tab stops and placeholders
    // Build snippet structure
}
```

**Snippet Expansion:**

```rust
pub struct SnippetExpander {
    snippet: Snippet,
    current_tab_stop: usize,
    values: HashMap<usize, String>,
}

impl SnippetExpander {
    pub fn insert(&self, buffer: &mut TextBuffer, position: usize) {
        // Insert snippet text
        // Create selections for first tab stop
    }

    pub fn next_tab_stop(&mut self) {
        // Move to next tab stop
        // Update selections
    }

    pub fn previous_tab_stop(&mut self) {
        // Move to previous tab stop
    }

    pub fn finish(&self) {
        // Jump to final position ($0)
        // Exit snippet mode
    }
}
```

**Snippet UI:**

1. **Expansion:**

   - Type trigger + Tab
   - Or select from completion menu
   - Enter snippet mode

2. **Snippet Mode:**

   - Highlight current placeholder
   - Tab: Next placeholder
   - Shift+Tab: Previous placeholder
   - Escape: Exit snippet mode
   - Edit placeholder: Update all occurrences (if mirrored)

3. **Choice Selection:**
   - Show dropdown for choices
   - Arrow keys to select
   - Enter to confirm

**Default Typst Snippets:**

````json
{
  "Document": {"prefix": "doc", "body": [...], "description": "Document template"},
  "Function": {"prefix": "fn", "body": [...], "description": "Function definition"},
  "Heading 1": {"prefix": "h1", "body": ["= $1"], "description": "Heading level 1"},
  "Heading 2": {"prefix": "h2", "body": ["== $1"], "description": "Heading level 2"},
  "List": {"prefix": "list", "body": ["- $1"], "description": "Unordered list"},
  "Numbered List": {"prefix": "nlist", "body": ["+ $1"], "description": "Ordered list"},
  "Table": {"prefix": "table", "body": [...], "description": "Table template"},
  "Math Block": {"prefix": "math", "body": ["$ $1 $"], "description": "Math block"},
  "Image": {"prefix": "img", "body": ["#image(\"$1\", width: ${2:50%})"], "description": "Insert image"},
  "Link": {"prefix": "link", "body": ["#link(\"$1\")[${2:text}]"], "description": "Hyperlink"},
  "Code Block": {"prefix": "code", "body": ["```$1", "$2", "```"], "description": "Code block"},
  "Bibliography": {"prefix": "bib", "body": [...], "description": "Bibliography entry"}
}
````

**User Snippets:**

- Allow users to define custom snippets
- Store in user config directory
- Merge with built-in snippets
- Support snippet scopes (global, Typst-only, etc.)

**Key Files:**

- `crates/editor_core/snippets/snippet.rs` - Snippet representation
- `crates/editor_core/snippets/parser.rs` - Snippet syntax parser
- `crates/editor_core/snippets/expander.rs` - Snippet expansion logic
- `crates/editor_core/snippets/library.rs` - Built-in snippets
- `crates/ui/editor/snippet_ui.rs` - Snippet UI integration
- `assets/snippets/typst.json` - Default Typst snippets
- `assets/snippets/markdown.json` - Markdown snippets (if supported)

### 8.4 Minimap

**Minimap Rendering:**

```rust
pub struct Minimap {
    pub width: f32,
    pub visible: bool,
}

impl Minimap {
    pub fn render(&self, buffer: &TextBuffer, viewport: &Viewport) -> Image {
        // Render simplified view of document
        // Show syntax highlighting (optional)
        // Render viewport indicator
    }

    fn render_line(&self, line: &str, y: f32) -> Vec<ColorBlock> {
        // Simplify line to color blocks
        // Ignore details, show general structure
    }
}

pub struct ColorBlock {
    pub x: f32,
    pub width: f32,
    pub color: Color,
}
```

**Minimap Interactions:**

- Click to scroll to position
- Drag viewport indicator to scroll
- Hover to show tooltip with line number

**Minimap Configuration:**

- Enable/disable minimap
- Minimap width (pixels)
- Show/hide syntax highlighting
- Scale factor (characters per pixel)

**Performance:**

- Update minimap on idle (debounced)
- Cache minimap image
- Invalidate cache on document changes
- Use low-resolution rendering

**Key Files:**

- `crates/ui/editor/minimap.rs` - Minimap rendering
- `crates/ui/editor/minimap_painter.rs` - Minimap drawing

### 8.5 Command Palette

**Command Registry:**

```rust
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}

pub struct Command {
    pub id: String,
    pub label: String,
    pub category: String,
    pub keybinding: Option<String>,
    pub action: Box<dyn Fn(&mut AppState) + Send + Sync>,
}

impl CommandRegistry {
    pub fn register(&mut self, command: Command) {
        self.commands.insert(command.id.clone(), command);
    }

    pub fn execute(&self, id: &str, state: &mut AppState) -> Result<()> {
        if let Some(cmd) = self.commands.get(id) {
            (cmd.action)(state);
            Ok(())
        } else {
            Err(Error::CommandNotFound)
        }
    }

    pub fn search(&self, query: &str) -> Vec<&Command> {
        // Fuzzy search commands
        // Return sorted by relevance
    }
}
```

**Command Palette UI:**

1. **Overlay:**

   - Centered overlay on top of editor
   - Input box with search query
   - Results list below
   - Keyboard-driven interface

2. **Search:**

   - Fuzzy matching on command labels
   - Show category and keybinding
   - Highlight matched characters
   - Sort by relevance and recent usage

3. **Navigation:**

   - Up/Down arrows: Navigate results
   - Enter: Execute selected command
   - Escape: Close palette
   - Type to filter

4. **Recent Commands:**
   - Show recently executed commands at top
   - Track command execution history

**Command Categories:**

- File
- Edit
- Selection
- View
- Go
- Tools
- Help

**Key Files:**

- `crates/ui/components/command_palette.rs` - Command palette UI
- `crates/ui/workspace/commands.rs` - Command registry
- `crates/ui/workspace/command_search.rs` - Fuzzy command search

### 8.6 Keyboard Shortcuts

**Keybinding System:**

```rust
pub struct KeybindingManager {
    bindings: HashMap<KeyBinding, String>,  // KeyBinding -> Command ID
    platform: Platform,
}

#[derive(Hash, Eq, PartialEq)]
pub struct KeyBinding {
    pub key: KeyCode,
    pub modifiers: Modifiers,
    pub when: Option<WhenCondition>,
}

pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub meta: bool,  // Cmd on macOS, Win on Windows
}

pub enum WhenCondition {
    EditorFocus,
    SearchActive,
    SnippetMode,
    // etc.
}

impl KeybindingManager {
    pub fn handle_key(&self, event: KeyEvent, state: &AppState) -> Option<String> {
        // Check when conditions
        // Find matching keybinding
        // Return command ID
    }
}
```

**Platform-Specific Bindings:**

Automatically translate between platforms:

- Ctrl on Windows/Linux â†” Cmd on macOS
- Alt on Windows/Linux â†” Option on macOS

**Default Keybindings:**

```json
{
	"file.new": { "key": "ctrl+n", "mac": "cmd+n" },
	"file.open": { "key": "ctrl+o", "mac": "cmd+o" },
	"file.save": { "key": "ctrl+s", "mac": "cmd+s" },
	"file.saveAs": { "key": "ctrl+shift+s", "mac": "cmd+shift+s" },
	"edit.undo": { "key": "ctrl+z", "mac": "cmd+z" },
	"edit.redo": { "key": "ctrl+y", "mac": "cmd+shift+z" },
	"edit.cut": { "key": "ctrl+x", "mac": "cmd+x" },
	"edit.copy": { "key": "ctrl+c", "mac": "cmd+c" },
	"edit.paste": { "key": "ctrl+v", "mac": "cmd+v" },
	"edit.find": { "key": "ctrl+f", "mac": "cmd+f" },
	"edit.replace": { "key": "ctrl+h", "mac": "cmd+alt+f" },
	"editor.action.addCursorAbove": { "key": "ctrl+alt+up", "mac": "cmd+alt+up" },
	"editor.action.addCursorBelow": {
		"key": "ctrl+alt+down",
		"mac": "cmd+alt+down"
	},
	"editor.action.selectNextOccurrence": { "key": "ctrl+d", "mac": "cmd+d" },
	"view.toggleSidebar": { "key": "ctrl+b", "mac": "cmd+b" },
	"view.togglePreview": { "key": "ctrl+k ctrl+v", "mac": "cmd+k cmd+v" },
	"view.commandPalette": { "key": "ctrl+shift+p", "mac": "cmd+shift+p" }
}
```

**Chord Keybindings:**

Support multi-key sequences (e.g., Ctrl+K, Ctrl+V):

- Track partial chord matches
- Show chord indicator
- Timeout for chord completion

**Customization:**

- Allow user-defined keybindings
- Keybinding editor in settings
- Conflict detection
- Import/export keybinding presets

**Key Files:**

- `crates/ui/workspace/keybindings.rs` - Keybinding system
- `crates/ui/workspace/actions.rs` - Action definitions
- `crates/ui/workspace/keybinding_parser.rs` - Parse keybinding config
- `crates/ui/settings/keybinding_editor.rs` - Keybinding configuration UI

### 8.7 Multi-Cursor Enhancements

**Advanced Multi-Cursor Operations:**

(Building on Phase 3.2)

1. **Add Cursor on Click:**

   - Alt+Click (Windows/Linux) or Option+Click (macOS)
   - Click to add cursor at position
   - Click existing cursor to remove it

2. **Add Cursor Above/Below:**

   - Ctrl/Cmd+Alt+Up: Add cursor on line above
   - Ctrl/Cmd+Alt+Down: Add cursor on line below
   - Align cursors to same column

3. **Select All Occurrences:**

   - Ctrl/Cmd+Shift+L: Add cursors at all occurrences of selection
   - Useful for batch renaming

4. **Column Selection:**

   - Alt+Shift+Drag: Create column selection
   - Creates cursors at each line
   - Useful for editing tables, aligned code

5. **Split Selection into Lines:**

   - Split multi-line selection into multiple cursors
   - One cursor per line

6. **Cursor Undo:**
   - Ctrl/Cmd+U: Undo last cursor addition
   - Allows refining cursor placement

**Visual Feedback:**

- Primary cursor: Solid color
- Secondary cursors: Slightly transparent
- All cursors blink in sync
- Show cursor count in status bar

**Key Files:**

- Previously defined in Phase 3.2
- Enhance existing implementations

---

## Phase 9: Settings & Preferences

### 9.1 Settings UI

**Settings Panel:**

```rust
pub struct SettingsPanel {
    pub categories: Vec<SettingsCategory>,
    pub search_query: String,
}

pub struct SettingsCategory {
    pub name: String,
    pub icon: String,
    pub sections: Vec<SettingsSection>,
}

pub struct SettingsSection {
    pub title: String,
    pub settings: Vec<Setting>,
}

pub enum Setting {
    Boolean { key: String, label: String, value: bool, default: bool },
    Number { key: String, label: String, value: i32, min: i32, max: i32, default: i32 },
    String { key: String, label: String, value: String, default: String },
    Choice { key: String, label: String, options: Vec<String>, selected: usize, default: usize },
    Color { key: String, label: String, value: Color, default: Color },
}
```

**Settings Categories:**

1. **Editor Settings:**

   - Font family
   - Font size
   - Line height
   - Tab size
   - Insert spaces (vs. tabs)
   - Word wrap
   - Line numbers
   - Minimap
   - Cursor style
   - Cursor blink
   - Auto-save
   - Auto-closing brackets
   - Auto-closing quotes

2. **Appearance:**

   - Theme selection
   - Icon theme
   - Font smoothing
   - UI scale
   - Sidebar position (left/right)

3. **LSP:**

   - Enable/disable LSP
   - LSP server path
   - Completion trigger characters
   - Hover delay
   - Signature help
   - LSP log level

4. **Compiler:**

   - Auto-compile on save
   - Auto-compile on change
   - Compilation delay (debounce)
   - Show compilation output

5. **Bidirectional Text:**

   - Enable bidi support
   - RTL line alignment
   - Math mode detection

6. **Keybindings:**
   - Customize keyboard shortcuts
   - Import/export keybinding presets

**Settings Search:**

- Search box at top of settings panel
- Filter settings by name or description
- Highlight search matches

**Settings Persistence:**

- Save settings to config file on change
- Auto-reload on external changes
- Validate settings before saving
- Show validation errors

**Settings UI Controls:**

- Checkbox for boolean
- Number input with increment/decrement buttons
- Text input for strings
- Dropdown for choices
- Color picker for colors
- Reset button for individual settings
- "Reset All" button for category

**Key Files:**

- `crates/ui/workspace/settings_panel.rs` - Settings UI
- `crates/editor_core/config/settings.rs` - Settings definitions
- `crates/editor_core/config/schema.rs` - Settings schema

### 9.2 Configuration Files

**Config File Format:**

Support both JSON and TOML:

```toml
# ~/.config/typst-studio/config.toml

[editor]
font_family = "Fira Code"
font_size = 14
line_height = 1.5
tab_size = 4
insert_spaces = true
word_wrap = false
line_numbers = true
minimap = true

[appearance]
theme = "dark"
ui_scale = 1.0

[lsp]
enabled = true
server_path = "typst-lsp"
hover_delay = 500

[compiler]
auto_compile_on_save = true
auto_compile_on_change = true
compilation_delay = 500

[keybindings]
"file.save" = "Ctrl+S"
"edit.find" = "Ctrl+F"
```

**Config Locations:**

1. **Global Config:**

   - Windows: `%APPDATA%\typst-studio\config.toml`
   - macOS: `~/Library/Application Support/typst-studio/config.toml`
   - Linux: `~/.config/typst-studio/config.toml`

2. **Workspace Config:**

   - `.typst-studio/config.toml` in workspace root
   - Overrides global settings for workspace

3. **Priority:** Workspace > Global > Defaults

**Config Loading:**

```rust
pub struct ConfigLoader {
    global_config: Config,
    workspace_config: Option<Config>,
}

impl ConfigLoader {
    pub fn load() -> Result<Config> {
        let mut config = Config::default();

        // Load global config
        if let Ok(global) = Self::load_file(Self::global_path()) {
            config.merge(global);
        }

        // Load workspace config
        if let Ok(workspace) = Self::load_file(Self::workspace_path()) {
            config.merge(workspace);
        }

        Ok(config)
    }

    pub fn watch(&mut self) -> Receiver<ConfigChange> {
        // Watch config files for changes
        // Return channel for notifications
    }
}
```

**Hot Reloading:**

- Watch config files for external changes
- Reload and merge on change
- Apply changes without restart (where possible)
- Notify user of config errors

**Config Validation:**

```rust
pub fn validate_config(config: &Config) -> Result<(), Vec<ConfigError>> {
    let mut errors = Vec::new();

    // Validate font size
    if config.editor.font_size < 6 || config.editor.font_size > 72 {
        errors.push(ConfigError::InvalidValue {
            key: "editor.font_size".to_string(),
            message: "Font size must be between 6 and 72".to_string(),
        });
    }

    // Validate tab size
    if config.editor.tab_size < 1 || config.editor.tab_size > 16 {
        errors.push(ConfigError::InvalidValue {
            key: "editor.tab_size".to_string(),
            message: "Tab size must be between 1 and 16".to_string(),
        });
    }

    // More validations...

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
```

**Import/Export:**

- Export current settings to file
- Import settings from file
- Share settings across machines

**Key Files:**

- `crates/editor_core/config/loader.rs` - Config file loading
- `crates/editor_core/config/validator.rs` - Config validation
- `crates/editor_core/config/watcher.rs` - File watching
- `crates/editor_core/config/schema.rs` - Config schema definition

---

## Phase 10: Polish & Performance

### 10.1 Performance Optimization

**Profiling:**

- Integrate Tracy or similar profiler
- Profile rendering, text operations, LSP communication
- Identify bottlenecks
- Set performance budgets

**Optimization Targets:**

1. **Text Rendering:**

   - GPU batching for text glyphs
   - Cache shaped text
   - Use texture atlas for glyphs
   - Minimize draw calls

2. **Buffer Operations:**

   - Optimize rope operations
   - Minimize allocations
   - Use copy-on-write where appropriate

3. **Bidi Text:**

   - Cache bidi analysis results
   - Invalidate only affected lines
   - Optimize reordering algorithm

4. **LSP Communication:**

   - Batch notifications where possible
   - Use incremental sync
   - Cancel outdated requests

5. **Compilation:**
   - Maximize incremental compilation
   - Cache parsed files
   - Parallel compilation where possible

**Virtual Scrolling:**

For very large files:

- Only render visible lines
- Pre-render lines just outside viewport
- Recycle line renderers

**Memory Management:**

- Monitor memory usage
- Set limits for caches
- Implement cache eviction policies
- Detect and fix memory leaks

**Performance Metrics:**

Target metrics:

- 60 FPS scrolling (< 16ms per frame)
- < 100ms keystroke latency
- < 500ms LSP response time
- < 2s compilation for medium documents (10-50 pages)

**Key Files:**

- `crates/editor_core/profiling/mod.rs` - Profiling integration
- `crates/editor_core/performance/metrics.rs` - Performance metrics collection

### 10.2 Error Handling & Logging

**Error Types:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("LSP error: {0}")]
    Lsp(String),

    #[error("Compilation error: {0}")]
    Compilation(String),

    #[error("Configuration error: {0}")]
    Config(String),

    // More error types...
}
```

**Graceful Degradation:**

- LSP failure â†’ Continue without LSP features
- Compiler failure â†’ Show error, keep old preview
- File watcher failure â†’ Fall back to manual refresh
- Never crash on user errors

**User-Friendly Messages:**

- Avoid technical jargon
- Suggest solutions
- Provide "Learn More" links
- Show relevant context

**Logging System:**

```rust
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

pub struct Logger {
    level: LogLevel,
    outputs: Vec<Box<dyn LogOutput>>,
}

pub trait LogOutput: Send + Sync {
    fn log(&self, level: LogLevel, message: &str);
}

// Log to file
// Log to console (debug builds)
// Log to UI (errors and warnings)
```

**Logging Configuration:**

- Set log level in settings
- Enable/disable file logging
- Log rotation and size limits
- Include logs in bug reports

**Crash Reporting:**

- Optional crash reporting (with user consent)
- Collect logs and system info
- Send to error tracking service
- Privacy-conscious (no document content)

**Key Files:**

- `crates/editor_core/error/mod.rs` - Error types
- `crates/editor_core/logging/mod.rs` - Logging system
- `crates/editor_core/logging/file_logger.rs` - File logging

### 10.3 Testing

**Test Categories:**

1. **Unit Tests:**

   - Buffer operations
   - Bidi algorithm
   - LSP protocol handling
   - Search/replace logic
   - Snippet parsing

2. **Integration Tests:**

   - Editor operations with buffer
   - LSP client with mock server
   - Compiler integration
   - File watching

3. **UI Tests:**

   - GPUI component testing
   - User interaction simulation
   - Layout calculations

4. **Performance Tests:**
   - Benchmark buffer operations
   - Benchmark rendering
   - Stress test with large files

**Test Fixtures:**

- Sample Typst documents
- RTL/bidi test documents
- Large documents (stress tests)
- Documents with errors

**Testing Strategy:**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_insert() {
        let mut buffer = RopeBuffer::new("hello");
        buffer.insert(5, " world");
        assert_eq!(buffer.text(), "hello world");
    }

    #[test]
    fn test_bidi_reordering() {
        let text = "Hello ×¢×‘×¨×™×ª world";
        let bidi = analyze_bidi(text, Direction::Ltr);
        // Assert correct visual ordering
    }

    #[tokio::test]
    async fn test_lsp_completion() {
        let mut client = MockLspClient::new();
        client.expect_completion(/* ... */);
        let result = client.request_completion(/* ... */).await;
        assert!(result.is_ok());
    }
}
```

**Continuous Integration:**

- Run tests on every commit
- Test on all platforms (Windows, macOS, Linux)
- Check code coverage
- Lint and format checks

**Key Files:**

- `tests/integration/buffer_tests.rs`
- `tests/integration/bidi_tests.rs`
- `tests/integration/lsp_tests.rs`
- `tests/integration/search_tests.rs`
- `tests/fixtures/` - Test documents

### 10.4 Documentation

**User Documentation:**

1. **Getting Started Guide:**

   - Installation instructions
   - First launch walkthrough
   - Creating first document
   - Basic editing
   - Compiling and previewing

2. **Feature Documentation:**

   - Multi-cursor editing
   - Search and replace
   - Code folding
   - Snippets
   - LSP features
   - Bidirectional text

3. **Keyboard Shortcuts Reference:**

   - Categorized shortcuts
   - Platform-specific (Windows, macOS, Linux)
   - Printable cheat sheet

4. **Settings Reference:**

   - All settings explained
   - Examples for common configurations

5. **Troubleshooting:**
   - Common issues and solutions
   - LSP not working
   - Compilation errors
   - Performance issues

**Developer Documentation:**

1. **Architecture Overview:**

   - High-level design
   - Component interaction
   - Threading model

2. **API Documentation:**

   - Generated from rustdoc comments
   - Examples for key APIs

3. **Contributing Guide:**

   - Setting up dev environment
   - Code style guidelines
   - Pull request process
   - Testing requirements

4. **Build Instructions:**
   - Platform-specific build steps
   - Dependencies
   - Packaging

**Documentation Format:**

- Markdown files in `/docs`
- In-app help viewer
- Online documentation site
- Searchable

**Key Files:**

- `docs/getting-started.md`
- `docs/features.md`
- `docs/keyboard-shortcuts.md`
- `docs/settings.md`
- `docs/troubleshooting.md`
- `docs/architecture.md`
- `docs/contributing.md`
- `README.md`

### 10.5 Accessibility

**Keyboard-Only Navigation:**

- All features accessible via keyboard
- Clear focus indicators
- Logical tab order
- Skip navigation links

**Screen Reader Support:**

- ARIA labels for UI elements
- Announce state changes
- Document structure navigation
- (Limited by GPUI's capabilities)

**Visual Accessibility:**

- High contrast themes
- Configurable font sizes
- Respect system font scaling
- Color-blind friendly color schemes
- Sufficient contrast ratios (WCAG 2.1)

**Focus Management:**

- Visible focus indicators
- Focus trap in modals
- Restore focus on dialog close
- Focus follows keyboard navigation

**Accessibility Testing:**

- Test with screen readers
- Test keyboard-only navigation
- Test with high contrast mode
- Test with different font sizes

**Key Files:**

- `crates/ui/accessibility/mod.rs` - Accessibility utilities
- `crates/ui/accessibility/focus.rs` - Focus management
- `crates/ui/accessibility/screen_reader.rs` - Screen reader support

---

## Implementation Order Summary

The phases should be implemented in the following order for optimal development flow:

### Recommended Implementation Sequence:

1. **Phase 2: UI Foundation (Weeks 1-4)**

   - Get visual structure in place
   - Enable visual testing from the start
   - Build with mock/stub backends

2. **Phase 3: Text Buffer & Editing (Weeks 5-6)**

   - Core editing functionality
   - Can test with UI from Phase 2

3. **Phase 5: LSP Integration (Weeks 7-9)**

   - Add language intelligence
   - Significantly improves editing experience
   - (Skip Phase 4 temporarily for faster progress)

4. **Phase 6: Compiler Integration (Weeks 10-11)**

   - Enable document compilation
   - Required for preview

5. **Phase 7: Preview Rendering (Weeks 12-14)**

   - Show compiled output
   - Complete basic editor loop

6. **Phase 4: Bidirectional Text (Weeks 15-17)**

   - Add RTL/bidi support
   - Build on solid foundation

7. **Phase 8: Advanced Features (Weeks 18-22)**

   - Search/replace
   - Code folding
   - Snippets
   - Command palette
   - Keyboard shortcuts

8. **Phase 9: Settings (Weeks 23-24)**

   - Make everything configurable
   - Polish user experience

9. **Phase 10: Polish & Performance (Weeks 25-28)**
   - Optimize performance
   - Comprehensive testing
   - Documentation
   - Final polish

### Milestone Checkpoints:

- **Milestone 1 (Week 4):** Basic UI with mock data
- **Milestone 2 (Week 6):** Functional text editor
- **Milestone 3 (Week 11):** Editor with compilation
- **Milestone 4 (Week 14):** Full editor with preview
- **Milestone 5 (Week 17):** RTL/bidi support complete
- **Milestone 6 (Week 22):** Feature-complete editor
- **Milestone 7 (Week 24):** Polished, configurable editor
- **Milestone 8 (Week 28):** Production-ready release

### Parallel Work Opportunities:

Some tasks can be worked on in parallel:

- UI design (Phase 2) can happen alongside architecture planning
- Documentation can be written as features are completed
- Testing can be written alongside implementation

### Critical Path:

The critical path is: Phase 2 â†’ Phase 3 â†’ Phase 6 â†’ Phase 7

These phases must be completed in order as each depends on the previous.

---

## Final Notes

This specification provides a comprehensive blueprint for implementing a full-featured Typst editor. Each phase builds on previous phases, and the order is designed to deliver value incrementally while maintaining a solid foundation.

Key design principles:

- **UI-first approach:** Visual testing from day one
- **Incremental development:** Each phase delivers working functionality
- **Proper integration points:** Clean interfaces between components
- **Performance-conscious:** Design for efficiency from the start
- **User-focused:** Features that improve the editing experience
- **Accessible:** Keyboard-driven, high contrast support
- **Cross-platform:** Windows, macOS, and Linux support

The estimated timeline of 28 weeks is for a dedicated team. Adjust based on team size, experience, and desired feature set. Some features (like advanced bidi support) can be deferred to later versions if needed.
