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

⚠️ **Early Development**: This project is in its initial setup phase. Core engine logic is not yet implemented.

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

Examples will be added as the project develops.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## Security

For security considerations, see [THREAT_MODEL.md](THREAT_MODEL.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Status

- [x] Project structure
- [x] Workspace setup
- [x] CI pipeline
- [ ] Core engine implementation
- [ ] HTML parsing
- [ ] CSS styling
- [ ] Layout engine
- [ ] JavaScript integration
- [ ] Rendering backend

## Community

- Issues: [GitHub Issues](https://github.com/yourusername/nomad-web-engine/issues)
- Discussions: [GitHub Discussions](https://github.com/yourusername/nomad-web-engine/discussions)
