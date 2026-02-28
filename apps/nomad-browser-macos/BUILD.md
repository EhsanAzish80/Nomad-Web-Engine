# Building the Nomad Browser for macOS

This guide walks you through building and running the Nomad Browser macOS application.

## Prerequisites

1. **macOS 13.0 or later**
2. **Xcode 14.0 or later**
3. **Rust toolchain** (install via [rustup](https://rustup.rs/))
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

## Quick Start

### Option 1: Command Line Build

```bash
# 1. Navigate to the app directory
cd apps/nomad-browser-macos

# 2. Build the Rust engine
chmod +x build_rust.sh
./build_rust.sh

# 3. The script will output the library location
# You can now open the project in Xcode
```

### Option 2: Manual Build

```bash
# 1. Build the Rust library
cd /path/to/Nomad-Web-Engine
cargo build --release -p nomad-c-api

# 2. Create lib directory
mkdir -p apps/nomad-browser-macos/lib

# 3. Copy libraries
cp target/release/libnomad_c_api.dylib apps/nomad-browser-macos/lib/
cp bindings/c-api/nomad_engine.h apps/nomad-browser-macos/NomadBrowser/
```

## Xcode Project Setup

Since we're providing source files, you'll need to create an Xcode project:

### Creating the Xcode Project

1. Open Xcode
2. Create a new project: File → New → Project
3. Choose "macOS" → "App"
4. Set:
   - Product Name: `NomadBrowser`
   - Team: Your team
   - Organization Identifier: com.yourname
   - Interface: SwiftUI
   - Language: Swift
5. Save in `apps/nomad-browser-macos/`

### Adding Files to the Project

1. **Add Swift Files:**
   - Drag all `.swift` files from `NomadBrowser/` into the project
   - Select "Copy items if needed"
   - Select "NomadBrowser" target

2. **Add Bridging Header:**
   - In Build Settings, set "Objective-C Bridging Header" to:
     ```
     NomadBrowser/NomadBrowser-Bridging-Header.h
     ```

3. **Link Rust Library:**
   - Select the NomadBrowser target
   - Go to "Build Phases" → "Link Binary with Libraries"
   - Click "+" and "Add Other..."
   - Navigate to `lib/libnomad_c_api.dylib`
   - Add it to the project

4. **Set Library Search Paths:**
   - In Build Settings, set "Library Search Paths" to:
     ```
     $(PROJECT_DIR)/lib
     ```

5. **Set Header Search Paths:**
   - In Build Settings, set "Header Search Paths" to:
     ```
     $(PROJECT_DIR)/NomadBrowser
     ```

6. **Enable App Transport Security:**
   - The Info.plist already includes the necessary settings

### Build and Run

1. Select the "NomadBrowser" scheme
2. Choose "My Mac" as the destination
3. Press ⌘R to build and run

## Project Structure

```
nomad-browser-macos/
├── NomadBrowser/
│   ├── NomadBrowserApp.swift      # App entry point
│   ├── ContentView.swift           # Main UI
│   ├── RenderView.swift            # Rendering view
│   ├── NomadEngine.swift           # Engine wrapper
│   ├── DisplayList.swift           # Display list types
│   ├── NomadBrowser-Bridging-Header.h
│   ├── nomad_engine.h              # C header (copied by build)
│   └── Info.plist
├── lib/
│   └── libnomad_c_api.dylib       # Rust library (built)
├── build_rust.sh                   # Build script
├── BUILD.md                        # This file
└── README.md                       # Overview
```

## Usage

1. Launch the app
2. Enter a URL (e.g., `example.com`)
3. Click "Go" or press Enter
4. The page will load and render as text
5. Click on blue underlined links to navigate


## Troubleshooting

### "libnomad_c_api.dylib not found"

**Solution:** Run the build script again:
```bash
./build_rust.sh
```

### "Undefined symbols for architecture arm64/x86_64"

**Solution:** Make sure you're building for the correct architecture:
```bash
# For Apple Silicon
cargo build --release --target aarch64-apple-darwin -p nomad-c-api

# For Intel
cargo build --release --target x86_64-apple-darwin -p nomad-c-api
```

### App crashes on launch

**Solution:** Check Console.app for error messages. Common issues:
- Library not found in the expected path
- Runtime library search path not set correctly
- Add `@rpath/libnomad_c_api.dylib` to "Runpath Search Paths"

### Cannot load HTTPS pages

**Solution:** The Info.plist should already allow arbitrary loads for development. For production, configure proper App Transport Security settings.

### Layout looks wrong

**Solution:** The viewport width defaults to 800px. You can adjust this in the code or by resizing the window (future enhancement).

## Development Tips

### Rebuilding the Rust Library

Every time you make changes to the Rust code:

```bash
cd apps/nomad-browser-macos
./build_rust.sh
```

Then rebuild in Xcode (⌘B).

### Debugging

To see Rust panics and errors:
1. In Xcode, go to Product → Scheme → Edit Scheme
2. Under "Run" → "Arguments"
3. Add environment variable:
   - Name: `RUST_BACKTRACE`
   - Value: `1`

### Performance

For better performance, always use Release builds of the Rust library:
```bash
cargo build --release -p nomad-c-api
```

## Known Limitations

- Single window only
- No tabs
- No history
- No bookmarks
- No image support
- No JavaScript
- No CSS styling (text only)
- Basic text layout

## Next Steps

See the main project README for roadmap and future enhancements.
