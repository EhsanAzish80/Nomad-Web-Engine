# Quick Start Guide

## Installation

### Prerequisites

Install Rust and Cargo via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Clone and Build

```bash
# Clone the repository
git clone https://github.com/yourusername/nomad-web-engine.git
cd nomad-web-engine

# Build the project
cargo build

# Run tests to verify
cargo test
```

## Basic Usage

### Using the Headless Demo

The simplest way to try the engine:

```bash
# Load a webpage and extract text
cargo run --example headless-demo -- https://example.com

# Try a more complex page
cargo run --example headless-demo -- https://en.wikipedia.org/wiki/Rust_(programming_language)
```

### Using the Engine in Your Code

Add to your `Cargo.toml`:

```toml
[dependencies]
nomad-core = { path = "path/to/nomad-web-engine/crates/nomad-core" }
```

Basic example:

```rust
use nomad_core::Engine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine with default settings
    let engine = Engine::new()?;
    
    // Load a URL and get the text
    let text = engine.load_url("https://example.com")?;
    
    println!("{}", text);
    
    Ok(())
}
```

### Custom Configuration

```rust
use nomad_core::{Engine, EngineConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Custom configuration
    let config = EngineConfig {
        max_html_size: 5 * 1024 * 1024,  // 5 MB
        max_dom_nodes: 50_000,             // 50k nodes
        timeout_secs: 15,                  // 15 seconds
    };
    
    let engine = Engine::with_config(config)?;
    let text = engine.load_url("https://example.com")?;
    
    println!("{}", text);
    
    Ok(())
}
```

### Error Handling

```rust
use nomad_core::{Engine, EngineError};

fn main() {
    let engine = match Engine::new() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Failed to create engine: {}", e);
            return;
        }
    };
    
    match engine.load_url("https://example.com") {
        Ok(text) => println!("Success:\n{}", text),
        Err(EngineError::Network(e)) => {
            eprintln!("Network error: {}", e);
        }
        Err(EngineError::Html(e)) => {
            eprintln!("HTML parsing error: {}", e);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Using Individual Components

### Network Layer Only

```rust
use nomad_net::NetworkLayer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let network = NetworkLayer::new()?;
    let response = network.fetch("https://example.com")?;
    
    println!("Final URL: {}", response.url);
    println!("Status: {}", response.status);
    println!("Body length: {} bytes", response.body.len());
    
    Ok(())
}
```

### HTML Parser Only

```rust
use nomad_html::HtmlParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let parser = HtmlParser::new();
    let html = r#"
        <html>
            <body>
                <h1>Hello</h1>
                <p>World!</p>
            </body>
        </html>
    "#;
    
    let dom = parser.parse(html)?;
    let text = dom.extract_text();
    
    println!("Text: {}", text);
    println!("Nodes: {}", dom.node_count());
    
    Ok(())
}
```

### Combined Example

```rust
use nomad_net::NetworkLayer;
use nomad_html::HtmlParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch HTML
    let network = NetworkLayer::new()?;
    let response = network.fetch("https://example.com")?;
    
    // Parse HTML
    let parser = HtmlParser::new();
    let dom = parser.parse(&response.body)?;
    
    // Extract text
    let text = dom.extract_text();
    
    println!("{}", text);
    
    Ok(())
}
```

## Troubleshooting

### Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Runtime Errors

**"Network error: Timeout"**
- Increase timeout in `EngineConfig`
- Check internet connection
- Verify the URL is accessible

**"Response too large"**
- Increase `max_html_size` in `EngineConfig`
- Or skip very large pages

**"DOM tree exceeds maximum node limit"**
- Increase `max_dom_nodes` in `EngineConfig`
- Or skip very complex pages

**"Invalid URL"**
- Ensure URL includes scheme (http:// or https://)
- Check URL format is correct
- Only HTTP and HTTPS are supported

## Development Tips

### Run with Debug Output

```bash
RUST_LOG=debug cargo run --example headless-demo -- https://example.com
```

### Run Specific Tests

```bash
# Test specific crate
cargo test -p nomad-net

# Test with output
cargo test -- --nocapture

# Test specific function
cargo test test_network_layer
```

### Check for Issues

```bash
# Check compilation
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

### Build Release Version

```bash
# Optimized build
cargo build --release

# Run release build
./target/release/examples/headless-demo https://example.com
```

## Performance Tips

1. **Use Release Builds**: Much faster than debug builds
   ```bash
   cargo run --release --example headless-demo -- https://example.com
   ```

2. **Adjust Limits**: Lower limits = less memory, faster parsing
   ```rust
   let config = EngineConfig {
       max_html_size: 1024 * 1024,  // 1 MB
       max_dom_nodes: 10_000,       // 10k nodes
       timeout_secs: 10,
   };
   ```

3. **Reuse Engine Instance**: Don't recreate for each URL
   ```rust
   let engine = Engine::new()?;
   for url in urls {
       let text = engine.load_url(url)?;
       // process text...
   }
   ```

## What's Next?

- **Milestone 1**: CSS parsing and styling
- **Milestone 2**: Layout engine
- **Milestone 3**: JavaScript execution
- **Milestone 4**: Rendering

See the [README](README.md) for the full roadmap.

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/yourusername/nomad-web-engine/issues)
- **Documentation**: Run `cargo doc --open`
- **Examples**: See `examples/` directory

## Contributing

Want to help build the engine? See [CONTRIBUTING.md](CONTRIBUTING.md)!
