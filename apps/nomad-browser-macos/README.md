# Nomad Browser macOS

A simple macOS browser application powered by the Nomad Web Engine.

## Features

- Single window browsing
- URL input bar
- Text rendering from Rust engine
- Clickable links
- Vertical scrolling

## Building

1. Build the Rust engine first:
```bash
cd ../..
cargo build --release
```

2. Open in Xcode:
```bash
open NomadBrowser.xcodeproj
```

3. Build and run (⌘R)

## Architecture

```
SwiftUI App
    ↓
C API Bridge
    ↓
Rust Engine (nomad-core)
    ↓
Display List (serialized bytes)
    ↓
CoreGraphics Rendering
```

## Requirements

- macOS 13.0+
- Xcode 14.0+
- Rust toolchain (for building the engine)
