# macOS Integration Guide

This document describes how the Nomad Web Engine integrates with the macOS SwiftUI application through the C API.

## Architecture Overview

```
┌─────────────────────────────────────┐
│     SwiftUI macOS Application       │
│  (apps/nomad-browser-macos/)        │
└───────────────┬─────────────────────┘
                │
                │ C ABI
                ▼
┌─────────────────────────────────────┐
│     C API (bindings/c-api/)         │
│  - nomad_engine_create()            │
│  - nomad_engine_load_url()          │
│  - nomad_engine_get_display_list()  │
│  - nomad_engine_tick()              │
└───────────────┬─────────────────────┘
                │
                │ Rust FFI
                ▼
┌─────────────────────────────────────┐
│     Rust Engine (crates/nomad-core) │
│  - Network layer (nomad-net)        │
│  - HTML parser (nomad-html)         │
│  - Layout engine                    │
│  - Display list generation          │
└─────────────────────────────────────┘
```

## Data Flow

### 1. Loading a URL

```
User enters URL in Swift UI
    ↓
NomadEngineWrapper.loadURL()
    ↓
nomad_engine_load_url() [C API]
    ↓
Engine.load_url() [Rust]
    ├─▶ NetworkLayer.fetch()
    ├─▶ HtmlParser.parse()
    ├─▶ layout_dom()
    └─▶ DisplayList generated
```

### 2. Rendering

```
Engine has display list
    ↓
nomad_engine_get_display_list() [C API]
    ↓
Returns ByteBuffer (JSON bytes)
    ↓
Swift decodes to DisplayList struct
    ↓
RenderView draws with CoreGraphics
```

### 3. Link Click

```
User clicks on text
    ↓
RenderView.mouseDown()
    ↓
Check DisplayList for link at point
    ↓
Call onLinkClick callback
    ↓
ContentView updates URL and loads
```

## C API Functions

### nomad_engine_create()

```c
NomadEngine* nomad_engine_create(void);
```

Creates a new engine instance.

**Returns:** Pointer to engine or NULL on failure

**Swift Usage:**
```swift
let engine = nomad_engine_create()
```

### nomad_engine_destroy()

```c
void nomad_engine_destroy(NomadEngine* engine);
```

Destroys an engine instance and frees memory.

**Swift Usage:**
```swift
nomad_engine_destroy(engine)
```

### nomad_engine_load_url()

```c
int32_t nomad_engine_load_url(NomadEngine* engine, const char* url);
```

Loads a URL in the engine.

**Parameters:**
- `engine`: Valid engine pointer
- `url`: Null-terminated URL string

**Returns:**
- `0`: Success
- `-1`: Invalid parameters
- `-2`: Invalid UTF-8 in URL
- `-3`: Load failed (network/parse error)

**Swift Usage:**
```swift
let result = urlString.withCString { urlPtr in
    nomad_engine_load_url(engine, urlPtr)
}
```

### nomad_engine_tick()

```c
void nomad_engine_tick(NomadEngine* engine);
```

Ticks the engine for animations/updates (currently a no-op).

**Swift Usage:**
```swift
nomad_engine_tick(engine)
```

### nomad_engine_get_display_list()

```c
ByteBuffer nomad_engine_get_display_list(const NomadEngine* engine);
```

Gets the current display list as JSON bytes.

**Returns:** ByteBuffer with data pointer and length

**Swift Usage:**
```swift
let buffer = nomad_engine_get_display_list(engine)
let data = Data(bytes: buffer.data!, count: buffer.len)
nomad_free_byte_buffer(buffer) // Important: free after use
```

### nomad_engine_set_viewport_width()

```c
void nomad_engine_set_viewport_width(NomadEngine* engine, float width);
```

Sets the viewport width for layout calculations.

**Swift Usage:**
```swift
nomad_engine_set_viewport_width(engine, 800.0)
```

### nomad_free_byte_buffer()

```c
void nomad_free_byte_buffer(ByteBuffer buffer);
```

Frees a byte buffer returned by the engine.

**Swift Usage:**
```swift
nomad_free_byte_buffer(buffer)
```

## Display List Format

The display list is serialized as JSON for easy cross-language compatibility.

### DisplayList Structure

```json
{
  "width": 800.0,
  "height": 1200.0,
  "items": [
    {
      "kind": {
        "Text": {
          "content": "Hello ",
          "font_size": 16.0,
          "is_link": false,
          "link_url": null
        }
      },
      "bounds": {
        "x": 20.0,
        "y": 20.0,
        "width": 50.0,
        "height": 16.0
      }
    }
  ]
}
```

### Swift Decoding

The Swift side uses `Codable` to decode:

```swift
struct DisplayList: Codable {
    let width: Float
    let height: Float
    let items: [DisplayItem]
}

struct DisplayItem: Codable {
    let kind: DisplayItemKind
    let bounds: Rect
}

enum DisplayItemKind: Codable {
    case text(TextItem)
}
```

## Memory Management

### Rust Side

- Engine is heap-allocated via `Box`
- Converted to raw pointer for C API
- Must be freed with `nomad_engine_destroy`

### Swift Side

- Engine pointer stored in `NomadEngineWrapper`
- Freed in `deinit` automatically
- ByteBuffers must be manually freed after use

### Safety Rules

1. **Never** use an engine pointer after calling `nomad_engine_destroy`
2. **Always** free ByteBuffers with `nomad_free_byte_buffer`
3. **Never** modify or free returned data directly (use provided functions)
4. **Check** for NULL pointers before dereferencing

## Thread Safety

### Current Implementation

- Engine is **NOT** thread-safe
- All calls must be from the same thread
- Swift wrapper uses `@MainActor` where appropriate

### Recommendations

- Always call C API from a consistent thread
- Use dispatch queues for async operations
- Update UI only on main thread

## Error Handling

### Rust Errors

Errors in Rust are caught and returned as error codes or NULL pointers.

### Swift Errors

```swift
@Published var error: String?

// Network/parse errors
if result != 0 {
    error = "Failed to load URL (error code: \(result))"
}

// Decoding errors
do {
    let list = try decoder.decode(DisplayList.self, from: data)
} catch {
    self.error = "Failed to decode: \(error.localizedDescription)"
}
```

## Performance Considerations

### Network I/O

- Currently blocking (synchronous)
- Should be called from background thread
- Swift wrapper uses `DispatchQueue.global()`

### Layout Calculation

- O(n) where n = number of DOM nodes
- Called synchronously after HTML parse
- Cached until viewport changes

### Rendering

- CoreGraphics drawing is fast for text
- Scales well to thousands of text items
- Consider view recycling for very large pages

### JSON Serialization

- JSON is slower than binary formats
- But simpler for cross-language use
- Consider switching to bincode/MessagePack for production

## Debugging

### Enabling Rust Logging

Set environment variable in Xcode:
```
RUST_BACKTRACE=1
RUST_LOG=debug
```

### Common Issues

**"Engine is NULL"**
- Check `nomad_engine_create` return value
- Verify library is linked correctly

**"Display list decoding failed"**
- Print raw JSON for inspection
- Check Rust serialization format matches Swift structs

**"Crashes on URL load"**
- Enable Rust backtrace
- Check for NULL pointer dereference
- Verify URL is valid UTF-8

## Future Enhancements

### Async API

Convert to async/await:
```swift
async func loadURL(_ url: String) async throws -> DisplayList
```

### Streaming

Stream display list updates as parsing progresses.

### WebAssembly

Support WebAssembly modules for JavaScript.

### GPU Rendering

Use Metal for hardware-accelerated rendering.

## Testing

### Unit Tests (Rust)

```bash
cargo test -p nomad-c-api
```

### Integration Tests (Swift)

```swift
func testEngineLifecycle() {
    let engine = nomad_engine_create()
    XCTAssertNotNil(engine)
    nomad_engine_destroy(engine)
}
```

### Manual Testing

1. Build Rust library: `./build_rust.sh`
2. Open in Xcode and run
3. Load test URLs
4. Verify rendering and link clicking

## Resources

- [C API Header](../../bindings/c-api/nomad_engine.h)
- [Swift Wrapper](NomadBrowser/NomadEngine.swift)
- [Display List Types](NomadBrowser/DisplayList.swift)
- [Render View](NomadBrowser/RenderView.swift)
