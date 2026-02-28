# Nomad Web Engine

A modern, multi-platform web engine written in Rust.

[![CI](https://github.com/yourusername/nomad-web-engine/workflows/CI/badge.svg)](https://github.com/yourusername/nomad-web-engine/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

## Overview

Nomad Web Engine is a modular, production-ready web rendering engine designed for:
- **Multi-platform support**: macOS, Linux, Windows, and more
- **Clean architecture**: Separated concerns across focused crates
- **FFI-friendly**: C API bindings for integration with other languages
- **Performance**: Written in Rust for safety and speed

🚀 **Milestone 0 Complete**: Basic HTML fetching and text extraction pipeline is functional!

## Architecture

The engine is organized as a Rust workspace with the following structure:

```
nomad-web-engine/
├── crates/          # Core engine crates
│   ├── nomad-core      # Foundation types and utilities
│   ├── nomad-net       # Network layer (HTTP/HTTPS)
│   ├── nomad-html      # HTML parsing and DOM
│   ├── nomad-style     # CSS parsing and styling
│   ├── nomad-layout    # Layout engine
│   ├── nomad-js        # JavaScript engine integration
│   └── nomad-render    # Rendering and painting
├── bindings/        # Language bindings
│   └── c-api          # C FFI interface
├── apps/            # Example applications
└── examples/        # Usage examples
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design decisions.

## Getting Started

### Prerequisites

- Rust 1.70+ (install via [rustup](https://rustup.rs/))
- Cargo (comes with Rust)

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/nomad-web-engine.git
cd nomad-web-engine

# Build all crates
cargo build

# Run tests
cargo test

# Build optimized release
cargo build --release
```

### Running Examples

#### Headless Demo (Milestone 0)

A simple CLI app that loads a URL and extracts readable text:

```bash
# Run the headless demo
cargo run --example headless-demo -- https://example.com

# Or build first
cargo build --example headless-demo
./target/debug/examples/headless-demo https://example.com
```

See [examples/headless-demo](examples/headless-demo/) for more details.

#### macOS Browser App

A native macOS browser with SwiftUI interface:

```bash
# Build the Rust engine
cd apps/nomad-browser-macos
./build_rust.sh

# Open in Xcode and build (⌘B) or run (⌘R)
# See apps/nomad-browser-macos/BUILD.md for detailed instructions
```

Features:
- Single window browsing
- URL input bar
- Text rendering from Rust engine
- Clickable links
- Vertical scrolling

## Contributing
### Milestone 0: Basic Reader ✅
- [x] Project structure and workspace setup
- [x] CI pipeline (macOS, Linux, Windows)
- [x] HTTP/HTTPS fetching with reqwest
- [x] HTML5 parsing with html5ever
- [x] DOM tree construction
- [x] Text extraction from visible elements
- [x] Resource limits (size, nodes, timeout)
- [x] Headless demo CLI application

### Future Milestones
- [ ] CSS parsing and computed styles
- [ ] Layout engine (box model, flexbox)
- [ ] JavaScript execution
- [ ] Rendering backend
- [ ] Full browser UI
This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Status

### Milestone 0: Basic Reader ✅
- [x] Project structure and workspace setup
- [x] CI pipeline (macOS, Linux, Windows)
- [x] HTTP/HTTPS fetching with reqwest
- [x] HTML5 parsing with html5ever
- [x] DOM tree construction
- [x] Text extraction from visible elements
- [x] Resource limits (size, nodes, timeout)
- [x] Headless demo CLI application

### Milestone 1: macOS Browser ✅
- [x] Display list and layout system
- [x] C API for FFI integration
- [x] Basic text layout with word wrapping
- [x] macOS SwiftUI application
- [x] URL input and navigation
- [x] Text rendering with CoreGraphics
- [x] Link detection and click handling
- [x] Vertical scrolling

### Future Milestones
- [ ] CSS parsing and computed styles
- [ ] Enhanced layout (flexbox, tables)
- [ ] JavaScript execution
- [ ] Image support
- [ ] Full rendering pipeline

## Community

- Issues: [GitHub Issues](https://github.com/yourusername/nomad-web-engine/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/nomad-web-engine/discussions)
