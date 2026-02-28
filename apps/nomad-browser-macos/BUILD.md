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

The Xcode project is already configured and ready to use.

### Build Configuration

The project includes:
- **Automatic library embedding**: The Rust library is automatically copied into the app bundle during build
- **Code signing**: The library is signed with the app's code signing identity
- **Bridging header**: Pre-configured to use the C API
- **Library search paths**: Already set to find the Rust library

### Adding Network Entitlements

To enable the app to load URLs from the internet:

1. Open the project in Xcode
2. Select the **NomadBrowser** target
3. Go to **Signing & Capabilities**
4. Under **App Sandbox**, enable:
   - ✅ **Outgoing Connections (Client)**

This is required for the app to make HTTP/HTTPS requests.

### Build and Run

1. Build the Rust library first:
   ```bash
   ./build_rust.sh
   ```

2. Open the project in Xcode:
   ```bash
   open NomadBrowser/NomadBrowser.xcodeproj
   ```

3. Select the "NomadBrowser" scheme
4. Choose "My Mac" as the destination
5. Press ⌘R to build and run

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

### "Library not loaded: @rpath/libnomad_c_api.dylib"

**Solution:** Run the build script to compile the Rust library:
```bash
./build_rust.sh
```

Then rebuild in Xcode (⌘B). The library will be automatically embedded in the app bundle.

### "Sandbox: deny file-write-create"

**Solution:** This is normal during development. The build system handles it automatically. If you see persistent issues, clean the build folder (⌘⇧K) and rebuild.

### App crashes on launch with "signal SIGABRT"

**Solution:** Check that:
- The Rust library was built successfully (`./build_rust.sh`)
- The library exists in `lib/libnomad_c_api.dylib`
- You've enabled network entitlements in Xcode (see above)

Run in Xcode with debugger to see detailed error messages.

### Cannot load URLs - "Network request failed"

**Solution:** Add network entitlements:
1. In Xcode, select the NomadBrowser target
2. Go to **Signing & Capabilities**
3. Under **App Sandbox**, enable:
   - ✅ **Outgoing Connections (Client)**

### "Code signature invalid"

**Solution:** Delete the app from DerivedData and rebuild:
```bash
rm -rf ~/Library/Developer/Xcode/DerivedData/NomadBrowser-*
```

Then rebuild in Xcode. The build system will automatically sign the library with the app's identity.

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
