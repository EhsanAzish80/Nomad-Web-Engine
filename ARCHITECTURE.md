# Architecture

## Design Philosophy

Nomad Web Engine follows these core principles:

1. **Modularity**: Each component is a separate crate with clear responsibilities
2. **Safety**: Leveraging Rust's ownership model and type system
3. **Performance**: Zero-cost abstractions and careful memory management
4. **Portability**: Cross-platform from day one
5. **Extensibility**: Clean APIs for bindings and embedders

## Crate Organization

### Core Crates (`crates/`)

#### `nomad-core`
**Purpose**: Foundation layer providing common types and utilities

**Responsibilities**:
- Core data structures
- Error types
- Logging infrastructure
- Platform abstractions

**Dependencies**: None (foundation crate)

---

#### `nomad-net`
**Purpose**: Network communication layer

**Responsibilities**:
- HTTP/HTTPS requests
- DNS resolution
- Connection pooling
- Certificate validation
- Protocol handling (HTTP/1.1, HTTP/2, HTTP/3)

**Dependencies**: `nomad-core`

---

#### `nomad-html`
**Purpose**: HTML parsing and DOM construction

**Responsibilities**:
- HTML5 tokenization
- Document tree building
- DOM API implementation
- Parser error handling

**Dependencies**: `nomad-core`

---

#### `nomad-style`
**Purpose**: CSS parsing and style computation

**Responsibilities**:
- CSS tokenization and parsing
- Cascade resolution
- Specificity calculation
- Computed style values

**Dependencies**: `nomad-core`

---

#### `nomad-layout`
**Purpose**: Layout computation

**Responsibilities**:
- Box model calculation
- Flow layout
- Flexbox
- Grid (future)
- Position calculation

**Dependencies**: `nomad-core`, `nomad-style`

---

#### `nomad-js`
**Purpose**: JavaScript engine integration

**Responsibilities**:
- JS runtime embedding
- DOM bindings
- Event handling
- API surface for web APIs

**Dependencies**: `nomad-core`

---

#### `nomad-render`
**Purpose**: Rendering and compositing

**Responsibilities**:
- Painting primitives
- Compositor
- GPU acceleration
- Text rendering

**Dependencies**: `nomad-core`, `nomad-layout`

---

### Bindings (`bindings/`)

#### `c-api`
**Purpose**: C-compatible FFI interface

**Responsibilities**:
- Opaque handle management
- C-style error handling
- Thread-safe API surface
- ABI stability considerations

**Type**: `cdylib` + `staticlib`

**Dependencies**: All core crates

---

## Data Flow

```
URL Request
    ↓
[nomad-net] ← Fetch HTML
    ↓
[nomad-html] ← Parse to DOM
    ↓
[nomad-style] ← Apply CSS
    ↓
[nomad-layout] ← Compute positions
    ↓
[nomad-render] ← Paint to screen
    ↑
[nomad-js] ← Handle events & mutations
```

## Thread Safety

- **Core types**: Send + Sync where appropriate
- **Resource loading**: Async-first with tokio
- **Rendering**: Main-thread bound with message passing
- **JS execution**: Isolated contexts per thread

## Memory Management

- **DOM nodes**: Arena allocation for cache locality
- **Style data**: Copy-on-write for shared rules
- **Layout boxes**: Pool allocation for reuse
- **FFI boundaries**: Explicit ownership transfer

## Platform Abstractions

Platform-specific code is isolated to `nomad-core`:
- Window system integration
- Font rendering backends
- Graphics API selection (Metal, Vulkan, DirectX)

## Future Considerations

- WebAssembly support
- Incremental layout
- CSS containment
- Web workers
- Service workers
- WebRTC
- WebGPU

## Testing Strategy

- **Unit tests**: Per-crate with comprehensive coverage
- **Integration tests**: Cross-crate interaction
- **Conformance tests**: Web Platform Tests (WPT)
- **Performance tests**: Benchmarking suite
- **Fuzzing**: Input validation and parser robustness

## Build Profiles

- **Debug**: Fast compilation, debug symbols
- **Release**: Full optimization, LTO enabled
- **Bench**: Profile-guided optimization
