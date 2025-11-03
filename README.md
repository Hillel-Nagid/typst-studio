# Typst Studio

A high-performance native cross-platform text editor for Typst documents with bidirectional text support, live preview capabilities, and full LSP integration.

## Features (Phase 1 - Foundation)

### Completed ✅

- **Workspace Architecture**: Modular crate structure for maintainability
- **Text Buffer**: Rope-based text buffer with efficient operations
- **Selection System**: Multi-cursor support with bidirectional text awareness
- **Edit Operations**: Undo/redo system with operation grouping
- **Bidirectional Text**: Full Unicode Bidirectional Algorithm (UAX #9) implementation
- **Typst Integration**: Compiler integration with world abstraction
- **Preview System**: Foundation for PDF/SVG preview rendering
- **LSP Client**: Protocol implementation for language server integration
- **State Management**: Application, workspace, and editor state architecture
- **UI Components**: Basic component structure for GPUI integration

## Project Structure

```
typst-studio/
├── crates/
│   ├── editor-core/          # Core editing logic
│   │   ├── buffer/           # Text buffer implementation
│   │   ├── selection/        # Selection and cursor management
│   │   └── operations/       # Edit operations (insert, delete, etc.)
│   ├── typst-integration/    # Typst compiler wrapper
│   │   ├── compiler/         # Compilation service
│   │   ├── diagnostics/      # Error handling
│   │   └── world/            # File system abstraction
│   ├── bidi-text/            # Bidirectional text handling
│   │   ├── algorithm/        # UAX #9 implementation
│   │   ├── layout/           # Visual layout engine
│   │   └── cursor/           # Bidi-aware cursor movement
│   ├── preview/              # Preview rendering
│   │   ├── renderer/         # PDF/SVG rendering
│   │   ├── sync/             # Source-preview synchronization
│   │   └── viewport/         # Viewport management
│   ├── lsp-client/           # LSP client implementation
│   │   ├── protocol/         # LSP protocol handlers
│   │   ├── requests/         # Request management
│   │   └── notifications/    # Notification handling
│   └── ui-components/        # Reusable UI components
├── assets/                   # Icons, themes, fonts
├── tests/                    # Integration tests
└── src/                      # Main application entry point
```

## Building

### Prerequisites

- Rust 1.75 or later
- GPUI dependencies (platform-specific)

### Build from source

```bash
# Clone the repository
git clone https://github.com/yourusername/typst-studio
cd typst-studio

# Build
cargo build --release

# Run
cargo run --release
```

### Development build

```bash
cargo build
cargo run
```

## Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p editor-core

# Run with logging
RUST_LOG=debug cargo test
```

## Roadmap

- [x] **Phase 1**: Foundation & Architecture (Current)
- [ ] **Phase 2**: Text Buffer with Bidirectional Support
- [ ] **Phase 3**: GPUI Editor View
- [ ] **Phase 4**: LSP Integration
- [ ] **Phase 5**: Typst Compilation and Preview
- [ ] **Phase 6**: Advanced Editor Features
- [ ] **Phase 7**: Polish and Optimization
- [ ] **Phase 8**: Release and Deployment

## Architecture

The editor is built using:

- **GPUI**: GPU-accelerated UI framework from Zed
- **Rope**: Efficient text buffer data structure via `ropey`
- **Unicode Bidi**: Full bidirectional text support via `unicode-bidi`
- **Typst**: Integration with Typst compiler and LSP
- **Tokio**: Async runtime for non-blocking operations

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is dual-licensed under MIT OR Apache-2.0.

## Acknowledgments

- [Typst](https://github.com/typst/typst) for the amazing typesetting system
- [Zed](https://github.com/zed-industries/zed) for the GPUI framework
- Unicode Consortium for the Bidirectional Algorithm specification
